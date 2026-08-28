use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

use self::model::{SolverResult, solve_fixture};
use super::{
    canonical_external_file, read_bounded_file, require_empty_output, resolve_cli_path,
    resolve_output_path, sha256_hex,
};

mod convergence;
mod model;

const MANIFEST_SCHEMA: &str = "nextengine.experimental-physical-sound-bem-feasibility.manifest.v1";
const REPORT_SCHEMA: &str = "nextengine.experimental-physical-sound-bem-feasibility.report.v1";
const STUDY_ID: &str = "physical-sound-surface-mode-bem-analytical-control";
const PROTOCOL_REVISION: &str = "pulsating-sphere-indirect-single-layer-v1";
const MANIFEST_SHA256: &str = "9a26ca137681b319b2ffb490dd1e4b897aeaf4de19349c7cbb669c97669c8095";
const MAX_MANIFEST_BYTES: usize = 128 * 1024;
static NEXT_STAGING: AtomicU64 = AtomicU64::new(0);

pub(super) fn run_cli(root: &Path, arguments: impl Iterator<Item = String>) -> Result<(), String> {
    let mut arguments = arguments.peekable();
    if arguments.peek().is_some_and(|value| value == "quadrature") {
        arguments.next();
        return convergence::run_cli(root, arguments);
    }
    let mut manifest = None;
    let mut output = None;
    while let Some(flag) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--manifest" => set_once(&mut manifest, PathBuf::from(value), &flag)?,
            "--output" => set_once(&mut output, PathBuf::from(value), &flag)?,
            _ => return Err(format!("unexpected bem-feasibility argument: {flag}")),
        }
    }
    let manifest = manifest.ok_or_else(|| {
        "physical-sound-registry bem-feasibility requires --manifest <external-json>".to_owned()
    })?;
    let output = output.ok_or_else(|| {
        "physical-sound-registry bem-feasibility requires --output <external-empty-directory>"
            .to_owned()
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
        "BEM feasibility manifest",
    )?;
    let manifest_bytes = read_bounded_file(
        &manifest_path,
        MAX_MANIFEST_BYTES,
        "BEM feasibility manifest",
    )?;
    let manifest_sha256 = sha256_hex(&manifest_bytes);
    if manifest_sha256 != MANIFEST_SHA256 {
        return Err(format!(
            "BEM feasibility manifest hash changed: expected {MANIFEST_SHA256}, got {manifest_sha256}"
        ));
    }
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("parse BEM feasibility manifest: {error}"))?;
    validate_manifest(&manifest)?;

    let solver = solve_fixture(&manifest.fixture, &manifest.solver)?;
    let report = build_report(&manifest, solver)?;
    let report_bytes = pretty_json(&report)?;
    let report_sha256 = sha256_hex(&report_bytes);
    let output = resolve_output_path(root, output_argument)?;
    require_empty_output(&output)?;
    publish(&output, &manifest_bytes, &report_bytes)?;

    println!("physical sound BEM feasibility: {}", output.display());
    println!("manifest sha256: {MANIFEST_SHA256}");
    println!("coarse panels: {}", report.coarse.panel_count);
    println!("fine panels: {}", report.fine.panel_count);
    println!(
        "fine max relative complex error: {:.9}",
        report.fine.max_relative_complex_error
    );
    println!(
        "fine max magnitude error dB: {:.9}",
        report.fine.max_absolute_magnitude_error_db
    );
    println!(
        "fine max phase error degrees: {:.9}",
        report.fine.max_absolute_phase_error_degrees
    );
    println!(
        "fine/coarse median error ratio: {:.9}",
        report.refinement.median_relative_complex_error_ratio
    );
    println!("decision: {}", report.decision);
    println!("report sha256: {report_sha256}");
    Ok(())
}

fn build_report(manifest: &Manifest, solver: SolverResult) -> Result<Report<'_>, String> {
    let coarse = aggregate_level(solver.coarse, &manifest.fixture)?;
    let fine = aggregate_level(solver.fine, &manifest.fixture)?;
    let ratio = fine.median_relative_complex_error / coarse.median_relative_complex_error;
    if !ratio.is_finite() {
        return Err("BEM refinement ratio is non-finite".to_owned());
    }
    let gates = GateReport {
        fine_relative_complex_error_passed: fine.max_relative_complex_error
            <= manifest.admission_gates.fine_max_relative_complex_error,
        fine_magnitude_error_passed: fine.max_absolute_magnitude_error_db
            <= manifest
                .admission_gates
                .fine_max_absolute_magnitude_error_db,
        fine_phase_error_passed: fine.max_absolute_phase_error_degrees
            <= manifest
                .admission_gates
                .fine_max_absolute_phase_error_degrees,
        fine_direction_symmetry_passed: fine.max_direction_magnitude_span_db
            <= manifest
                .admission_gates
                .fine_max_direction_magnitude_span_db,
        refinement_passed: ratio
            <= manifest
                .admission_gates
                .fine_to_coarse_median_relative_complex_error_ratio,
    };
    let passed = gates.all_passed();
    Ok(Report {
        schema: REPORT_SCHEMA,
        status: "Validated",
        decision: if passed {
            "ClassicalBoundarySolverAnalyticalControlSupported"
        } else {
            "ClassicalBoundarySolverAnalyticalControlRejected"
        },
        claim: "ANALYTICAL_PULSATING_SPHERE_BOUNDARY_SOLVER_FEASIBILITY_ONLY / NO_REAL_OBJECT_MATERIAL_QUALITY_ADMISSION_RUNTIME_OR_REALIMPACT_CREDIT",
        study_id: STUDY_ID,
        protocol_revision: PROTOCOL_REVISION,
        manifest_sha256: MANIFEST_SHA256,
        coarse,
        fine,
        refinement: RefinementReport {
            median_relative_complex_error_ratio: ratio,
        },
        thresholds: &manifest.admission_gates,
        gate: gates,
        data_policy: &manifest.data_policy,
        source_lineage: &manifest.source_lineage,
        next_action: if passed {
            "freeze one synthetic non-spherical surface-mode fixture and compare this solver with an independent classical BEM implementation before opening any fresh REALIMPACT payload"
        } else {
            "keep REALIMPACT sealed; diagnose the boundary equation, quadrature or mesh convergence against the same analytical fixture before any new representation work"
        },
    })
}

fn aggregate_level(level: model::LevelResult, fixture: &Fixture) -> Result<LevelReport, String> {
    let mut relative_errors = level
        .conditions
        .iter()
        .map(|condition| condition.relative_complex_error)
        .collect::<Vec<_>>();
    if relative_errors.is_empty() || relative_errors.iter().any(|value| !value.is_finite()) {
        return Err("BEM level has invalid condition errors".to_owned());
    }
    relative_errors.sort_by(f64::total_cmp);
    let median_relative_complex_error = median(&relative_errors);
    let max_relative_complex_error = *relative_errors
        .last()
        .ok_or_else(|| "BEM level has no maximum error".to_owned())?;
    let max_absolute_magnitude_error_db = level
        .conditions
        .iter()
        .map(|condition| condition.absolute_magnitude_error_db)
        .max_by(f64::total_cmp)
        .ok_or_else(|| "BEM level has no magnitude errors".to_owned())?;
    let max_absolute_phase_error_degrees = level
        .conditions
        .iter()
        .map(|condition| condition.absolute_phase_error_degrees)
        .max_by(f64::total_cmp)
        .ok_or_else(|| "BEM level has no phase errors".to_owned())?;
    let max_direction_magnitude_span_db = direction_span(&level, fixture)?;
    Ok(LevelReport {
        subdivision_level: level.subdivision_level,
        panel_count: level.panel_count,
        condition_count: level.conditions.len(),
        median_relative_complex_error,
        max_relative_complex_error,
        max_absolute_magnitude_error_db,
        max_absolute_phase_error_degrees,
        max_direction_magnitude_span_db,
        conditions: level.conditions,
    })
}

fn direction_span(level: &model::LevelResult, fixture: &Fixture) -> Result<f64, String> {
    let mut maximum = 0.0_f64;
    for wave_number_radius in &fixture.wave_number_radius_values {
        for listener_radius_multiplier in &fixture.listener_radius_multipliers {
            let magnitudes = level
                .conditions
                .iter()
                .filter(|condition| {
                    condition.wave_number_radius == *wave_number_radius
                        && condition.listener_radius_multiplier == *listener_radius_multiplier
                })
                .map(|condition| 20.0 * condition.computed.magnitude().log10())
                .collect::<Vec<_>>();
            if magnitudes.len() != fixture.listener_directions.len() {
                return Err("BEM direction group is incomplete".to_owned());
            }
            let minimum = magnitudes
                .iter()
                .copied()
                .min_by(f64::total_cmp)
                .ok_or_else(|| "BEM direction group is empty".to_owned())?;
            let maximum_group = magnitudes
                .iter()
                .copied()
                .max_by(f64::total_cmp)
                .ok_or_else(|| "BEM direction group is empty".to_owned())?;
            maximum = maximum.max(maximum_group - minimum);
        }
    }
    maximum
        .is_finite()
        .then_some(maximum)
        .ok_or_else(|| "BEM direction span is non-finite".to_owned())
}

fn median(sorted: &[f64]) -> f64 {
    if sorted.len().is_multiple_of(2) {
        (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) * 0.5
    } else {
        sorted[sorted.len() / 2]
    }
}

fn validate_manifest(manifest: &Manifest) -> Result<(), String> {
    if manifest.schema != MANIFEST_SCHEMA
        || manifest.study_id != STUDY_ID
        || manifest.protocol_revision != PROTOCOL_REVISION
        || manifest.purpose
            != "Test whether a deterministic classical boundary solver can recover an analytical complex radiation field before any further REALIMPACT access."
        || manifest.fixture.id != "unit-normal-derivative-pulsating-sphere"
        || manifest.fixture.radius_metres != 0.1
        || manifest.fixture.speed_of_sound_metres_per_second != 343.0
        || manifest.fixture.air_density_kilograms_per_cubic_metre != 1.225
        || manifest.fixture.normal_pressure_derivative_real != 1.0
        || manifest.fixture.normal_pressure_derivative_imaginary != 0.0
        || manifest.fixture.wave_number_radius_values != [0.25, 0.75, 1.5]
        || manifest.fixture.listener_radius_multipliers != [1.5, 3.0, 10.0]
        || manifest.fixture.listener_directions.len() != 10
        || manifest.solver.family != "indirect-single-layer-helmholtz-boundary-element"
        || manifest.solver.green_function != "exp(i*k*r)/(4*pi*r)"
        || manifest.solver.boundary_equation
            != "(-0.5*I + K_prime)*sigma = prescribed_normal_pressure_derivative"
        || manifest.solver.surface_mesh != "outward-oriented-icosphere"
        || manifest.solver.constant_panel_subdivision_levels != [1, 2]
        || manifest.solver.off_diagonal_panel_quadrature != "symmetric-three-point-triangle"
        || manifest.solver.diagonal_jump_term != -0.5
        || manifest.solver.linear_solver
            != "deterministic-complex-gaussian-elimination-with-partial-pivoting"
        || manifest.solver.pivot_floor != 1.0e-12
        || manifest.source_lineage.len() != 2
        || !manifest.data_policy.generated_analytical_fixture_only
        || manifest.data_policy.fresh_realimpact_payload_access_allowed
        || manifest.data_policy.network_training_allowed
        || manifest
            .data_policy
            .runtime_or_quality_admission_credit_allowed
        || !manifest.admission_gates.repeat_report_bytes_identical
    {
        return Err("BEM feasibility manifest does not match the frozen protocol".to_owned());
    }
    for direction in &manifest.fixture.listener_directions {
        let norm = direction
            .iter()
            .map(|value| value * value)
            .sum::<f64>()
            .sqrt();
        if direction.iter().any(|value| !value.is_finite()) || norm <= 0.0 {
            return Err("BEM listener direction is invalid".to_owned());
        }
    }
    Ok(())
}

fn pretty_json(value: &impl Serialize) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("serialize BEM feasibility report: {error}"))?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn publish(output: &Path, manifest: &[u8], report: &[u8]) -> Result<(), String> {
    let parent = output
        .parent()
        .ok_or_else(|| "BEM feasibility output has no parent".to_owned())?;
    let sequence = NEXT_STAGING.fetch_add(1, Ordering::Relaxed);
    let staging = parent.join(format!(
        ".nextengine-bem-feasibility-{}-{sequence}",
        std::process::id()
    ));
    fs::create_dir(&staging).map_err(|error| format!("create BEM staging: {error}"))?;
    let result = (|| {
        fs::write(staging.join("manifest.json"), manifest)
            .map_err(|error| format!("write BEM manifest: {error}"))?;
        fs::write(staging.join("report.json"), report)
            .map_err(|error| format!("write BEM report: {error}"))?;
        if output.exists() {
            fs::remove_dir(output)
                .map_err(|error| format!("remove confirmed-empty BEM output: {error}"))?;
        }
        fs::rename(&staging, output).map_err(|error| format!("publish BEM output: {error}"))
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Manifest {
    schema: String,
    study_id: String,
    protocol_revision: String,
    purpose: String,
    fixture: Fixture,
    solver: Solver,
    admission_gates: AdmissionGates,
    source_lineage: Vec<SourceLineage>,
    data_policy: DataPolicy,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Fixture {
    id: String,
    pub(super) radius_metres: f64,
    pub(super) speed_of_sound_metres_per_second: f64,
    air_density_kilograms_per_cubic_metre: f64,
    pub(super) normal_pressure_derivative_real: f64,
    pub(super) normal_pressure_derivative_imaginary: f64,
    pub(super) wave_number_radius_values: Vec<f64>,
    pub(super) listener_radius_multipliers: Vec<f64>,
    pub(super) listener_directions: Vec<[f64; 3]>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Solver {
    family: String,
    green_function: String,
    boundary_equation: String,
    surface_mesh: String,
    pub(super) constant_panel_subdivision_levels: Vec<usize>,
    off_diagonal_panel_quadrature: String,
    diagonal_jump_term: f64,
    linear_solver: String,
    pub(super) pivot_floor: f64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct AdmissionGates {
    fine_max_relative_complex_error: f64,
    fine_max_absolute_magnitude_error_db: f64,
    fine_max_absolute_phase_error_degrees: f64,
    fine_max_direction_magnitude_span_db: f64,
    fine_to_coarse_median_relative_complex_error_ratio: f64,
    repeat_report_bytes_identical: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SourceLineage {
    id: String,
    revision: String,
    url: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct DataPolicy {
    generated_analytical_fixture_only: bool,
    fresh_realimpact_payload_access_allowed: bool,
    network_training_allowed: bool,
    runtime_or_quality_admission_credit_allowed: bool,
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
    coarse: LevelReport,
    fine: LevelReport,
    refinement: RefinementReport,
    thresholds: &'a AdmissionGates,
    gate: GateReport,
    data_policy: &'a DataPolicy,
    source_lineage: &'a [SourceLineage],
    next_action: &'static str,
}

#[derive(Debug, Serialize)]
struct LevelReport {
    subdivision_level: usize,
    panel_count: usize,
    condition_count: usize,
    median_relative_complex_error: f64,
    max_relative_complex_error: f64,
    max_absolute_magnitude_error_db: f64,
    max_absolute_phase_error_degrees: f64,
    max_direction_magnitude_span_db: f64,
    conditions: Vec<model::ConditionResult>,
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
}

impl GateReport {
    fn all_passed(&self) -> bool {
        self.fine_relative_complex_error_passed
            && self.fine_magnitude_error_passed
            && self.fine_phase_error_passed
            && self.fine_direction_symmetry_passed
            && self.refinement_passed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_manifest_hash_is_lowercase_hex() {
        assert_eq!(MANIFEST_SHA256.len(), 64);
        assert!(
            MANIFEST_SHA256
                .bytes()
                .all(|byte| { byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte) })
        );
    }

    #[test]
    fn median_handles_even_and_odd_counts() {
        assert_eq!(median(&[1.0, 2.0, 3.0]), 2.0);
        assert_eq!(median(&[1.0, 2.0, 3.0, 4.0]), 2.5);
    }
}
