use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

use self::model::{ComplexValue, evaluate, fit_coefficients, multipole_basis, normalized_cosine};
use super::super::{
    canonical_external_file, read_bounded_file, require_empty_output, resolve_cli_path,
    resolve_output_path, sha256_hex,
};

mod model;

const MANIFEST_SCHEMA: &str =
    "nextengine.experimental-physical-sound-surface-mode-cooker.manifest.v1";
const REPORT_SCHEMA: &str = "nextengine.experimental-physical-sound-surface-mode-cooker.report.v1";
const STUDY_ID: &str = "physical-sound-outgoing-multipole-cooker-quadrupole-control";
const PROTOCOL_REVISION: &str = "axisymmetric-outgoing-multipole-order3-v1";
const MANIFEST_SHA256: &str = "e76d82cb9194cb0c3c8d8b8489825e29c03ab5b7e5038b8b68d68b9e16d398fc";
const SOURCE_MANIFEST_SHA256: &str =
    "3a67f4add95dfc08fe7f55a08d9f6372fa478c1712a5c375910e23c1e2650e39";
const SOURCE_REPORT_SHA256: &str =
    "e8e1d4d594c0062979dafe6bcf3b29e72f3125f4e272e29dd41b7fb3d8816437";
const MAX_MANIFEST_BYTES: usize = 128 * 1024;
const MAX_SOURCE_REPORT_BYTES: usize = 512 * 1024;
static NEXT_STAGING: AtomicU64 = AtomicU64::new(0);

pub(super) fn run_cli(root: &Path, arguments: impl Iterator<Item = String>) -> Result<(), String> {
    let mut manifest = None;
    let mut output = None;
    let mut arguments = arguments;
    while let Some(flag) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--manifest" => set_once(&mut manifest, PathBuf::from(value), &flag)?,
            "--output" => set_once(&mut output, PathBuf::from(value), &flag)?,
            _ => return Err(format!("unexpected surface-mode-cooker argument: {flag}")),
        }
    }
    let manifest = manifest.ok_or_else(|| {
        "physical-sound-registry bem-feasibility surface-mode-cooker requires --manifest <external-json>"
            .to_owned()
    })?;
    let output = output.ok_or_else(|| {
        "physical-sound-registry bem-feasibility surface-mode-cooker requires --output <external-empty-directory>"
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
        "surface-mode cooker manifest",
    )?;
    let manifest_bytes = read_bounded_file(
        &manifest_path,
        MAX_MANIFEST_BYTES,
        "surface-mode cooker manifest",
    )?;
    require_hash(
        &manifest_bytes,
        MANIFEST_SHA256,
        "surface-mode cooker manifest",
    )?;
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("parse surface-mode cooker manifest: {error}"))?;
    validate_manifest(&manifest)?;
    let base = manifest_path
        .parent()
        .ok_or_else(|| "surface-mode cooker manifest has no parent".to_owned())?;
    let source_manifest_bytes = read_reference(
        root,
        base,
        &manifest.source_manifest,
        MAX_MANIFEST_BYTES,
        "Bempp surface-mode source manifest",
    )?;
    let source_report_bytes = read_reference(
        root,
        base,
        &manifest.source_report,
        MAX_SOURCE_REPORT_BYTES,
        "Bempp surface-mode source report",
    )?;
    let source_manifest: SourceManifest = serde_json::from_slice(&source_manifest_bytes)
        .map_err(|error| format!("parse Bempp surface-mode source manifest: {error}"))?;
    let source_report: SourceReport = serde_json::from_slice(&source_report_bytes)
        .map_err(|error| format!("parse Bempp surface-mode source report: {error}"))?;
    validate_source(&manifest, &source_manifest, &source_report)?;

    let report = build_report(&manifest, &source_manifest, &source_report)?;
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

    println!("physical sound surface-mode cooker: {}", output.display());
    println!("manifest sha256: {MANIFEST_SHA256}");
    println!("source report sha256: {SOURCE_REPORT_SHA256}");
    println!(
        "minimum degree-two energy fraction: {:.9}",
        report
            .aggregate
            .minimum_degree_two_coefficient_energy_fraction
    );
    println!(
        "held max peak-normalized error: {:.9}",
        report.aggregate.held_max_peak_normalized_complex_error
    );
    println!(
        "held minimum directional correlation: {:.9}",
        report.aggregate.held_min_directional_complex_correlation
    );
    println!("decision: {}", report.decision);
    println!("report sha256: {report_sha256}");
    Ok(())
}

fn read_reference(
    root: &Path,
    base: &Path,
    reference: &ArtifactReference,
    maximum_bytes: usize,
    label: &str,
) -> Result<Vec<u8>, String> {
    let path = canonical_external_file(root, &base.join(&reference.path), label)?;
    let bytes = read_bounded_file(&path, maximum_bytes, label)?;
    require_hash(&bytes, &reference.sha256, label)?;
    Ok(bytes)
}

fn require_hash(bytes: &[u8], expected: &str, label: &str) -> Result<(), String> {
    let actual = sha256_hex(bytes);
    if actual != expected {
        return Err(format!(
            "{label} hash changed: expected {expected}, got {actual}"
        ));
    }
    Ok(())
}

fn validate_manifest(manifest: &Manifest) -> Result<(), String> {
    if manifest.schema != MANIFEST_SCHEMA
        || manifest.study_id != STUDY_ID
        || manifest.protocol_revision != PROTOCOL_REVISION
        || manifest.source_manifest.path != Path::new("run-a/manifest.json")
        || manifest.source_manifest.sha256 != SOURCE_MANIFEST_SHA256
        || manifest.source_report.path != Path::new("run-a/report.json")
        || manifest.source_report.sha256 != SOURCE_REPORT_SHA256
        || manifest.source_contract.schema
            != "nextengine.experimental-physical-sound-bempp-surface-mode.report.v1"
        || manifest.source_contract.study_id
            != "physical-sound-independent-bempp-quadrupole-surface-mode"
        || manifest.source_contract.protocol_revision != "bempp-direct-neumann-to-dirichlet-p2-v3"
        || manifest.source_contract.manifest_sha256 != SOURCE_MANIFEST_SHA256
        || manifest.source_contract.decision != "IndependentBemppQuadrupoleSurfaceModeSupported"
        || manifest.source_contract.mesh_refinement_level != 3
        || manifest.source_contract.panel_count != 512
        || manifest.source_contract.condition_count != 198
        || manifest.split.fit_listener_radius_multiplier != 1.5
        || manifest.split.fit_direction_indices != [0, 1, 2, 6, 10, 14, 18]
        || manifest.split.held_listener_radius_multipliers != [3.0, 10.0]
        || manifest.split.held_direction_indices != (0..22).collect::<Vec<_>>()
        || manifest.candidate.id != "axisymmetric-complex-outgoing-multipole-order3-ridge1e12-v1"
        || manifest.candidate.maximum_order != 3
        || manifest.candidate.ridge != 1.0e-12
        || manifest.candidate.speed_of_sound_metres_per_second != 343.0
        || manifest.candidate.radial_basis != "spherical-hankel-first-kind"
        || manifest.candidate.angular_basis != "legendre-order-zero-through-three"
        || manifest.candidate.coefficient_fit != "complex-ridge-normal-equations-partial-pivot"
        || !manifest.admission_gates.require_every_source_gate_passed
        || !manifest.admission_gates.repeat_report_bytes_identical
        || manifest.allowed_claims
            != [
                "synthetic_quadrupole_surface_mode_cooker_sufficiency",
                "near_to_far_axisymmetric_outgoing_multipole_transfer",
            ]
        || manifest.prohibited_claims.len() != 7
        || !manifest.data_policy.generated_analytical_fixture_only
        || manifest.data_policy.fresh_realimpact_payload_access_allowed
        || manifest.data_policy.network_training_allowed
        || manifest
            .data_policy
            .runtime_or_quality_admission_credit_allowed
    {
        return Err("surface-mode cooker manifest does not match the frozen protocol".to_owned());
    }
    Ok(())
}

fn validate_source(
    manifest: &Manifest,
    source_manifest: &SourceManifest,
    source_report: &SourceReport,
) -> Result<(), String> {
    let contract = &manifest.source_contract;
    if source_manifest.schema
        != "nextengine.experimental-physical-sound-bempp-surface-mode.manifest.v1"
        || source_manifest.study_id != contract.study_id
        || source_manifest.protocol_revision != contract.protocol_revision
        || source_manifest.fixture.radius_metres != 0.1
        || source_manifest.fixture.speed_of_sound_metres_per_second != 343.0
        || source_manifest.fixture.spherical_harmonic_degree != 2
        || source_manifest.fixture.spherical_harmonic_order != 0
        || source_manifest.fixture.angular_profile != "legendre_p2_z"
        || source_manifest.fixture.wave_number_radius_values != [0.25, 0.75, 1.5]
        || source_manifest.fixture.listener_radius_multipliers != [1.5, 3.0, 10.0]
        || source_manifest.fixture.listener_directions.len() != 22
        || source_report.schema != contract.schema
        || source_report.study_id != contract.study_id
        || source_report.protocol_revision != contract.protocol_revision
        || source_report.manifest_sha256 != contract.manifest_sha256
        || source_report.decision != contract.decision
        || source_report.fine.mesh_refinement_level != contract.mesh_refinement_level
        || source_report.fine.panel_count != contract.panel_count
        || source_report.fine.condition_count != contract.condition_count
        || source_report.fine.conditions.len() != contract.condition_count
        || source_report.gate.is_empty()
        || source_report.gate.values().any(|passed| !passed)
    {
        return Err("Bempp surface-mode source does not match the frozen contract".to_owned());
    }
    for condition in &source_report.fine.conditions {
        if !condition.is_finite()
            || !source_manifest
                .fixture
                .wave_number_radius_values
                .contains(&condition.wave_number_radius)
            || !source_manifest
                .fixture
                .listener_radius_multipliers
                .contains(&condition.listener_radius_multiplier)
            || condition.direction_index >= source_manifest.fixture.listener_directions.len()
        {
            return Err("Bempp surface-mode source condition is invalid".to_owned());
        }
    }
    for wave_number_radius in &source_manifest.fixture.wave_number_radius_values {
        for radius_multiplier in &source_manifest.fixture.listener_radius_multipliers {
            for direction_index in 0..source_manifest.fixture.listener_directions.len() {
                let count = source_report
                    .fine
                    .conditions
                    .iter()
                    .filter(|condition| {
                        condition.wave_number_radius == *wave_number_radius
                            && condition.listener_radius_multiplier == *radius_multiplier
                            && condition.direction_index == direction_index
                    })
                    .count();
                if count != 1 {
                    return Err("Bempp surface-mode source condition grid is incomplete".to_owned());
                }
            }
        }
    }
    Ok(())
}

fn build_report<'a>(
    manifest: &'a Manifest,
    source_manifest: &SourceManifest,
    source_report: &SourceReport,
) -> Result<Report<'a>, String> {
    let mut frequencies = Vec::new();
    let mut held_conditions = Vec::new();
    for wave_number_radius in &source_manifest.fixture.wave_number_radius_values {
        let fit_rows = select_conditions(
            &source_report.fine.conditions,
            *wave_number_radius,
            manifest.split.fit_listener_radius_multiplier,
            &manifest.split.fit_direction_indices,
        )?;
        let fit = fit_coefficients(
            &fit_rows,
            source_manifest,
            *wave_number_radius,
            manifest.candidate.ridge,
        )?;
        let total_energy = fit
            .coefficients
            .iter()
            .map(|value| value.magnitude_squared())
            .sum::<f64>();
        if !total_energy.is_finite() || total_energy <= 1.0e-30 {
            return Err("surface-mode cooker coefficient energy is invalid".to_owned());
        }
        let degree_two_fraction = fit.coefficients[2].magnitude_squared() / total_energy;
        frequencies.push(FrequencyReport {
            wave_number_radius: *wave_number_radius,
            frequency_hz: fit_rows[0].frequency_hz,
            fit_condition_count: fit_rows.len(),
            coefficient_energy_fraction_by_degree: fit
                .coefficients
                .iter()
                .map(|value| value.magnitude_squared() / total_energy)
                .collect(),
            degree_two_coefficient_energy_fraction: degree_two_fraction,
            coefficients: fit.coefficients.to_vec(),
            fit_max_peak_normalized_complex_error: fit.max_peak_normalized_complex_error,
        });
        for radius_multiplier in &manifest.split.held_listener_radius_multipliers {
            let rows = select_conditions(
                &source_report.fine.conditions,
                *wave_number_radius,
                *radius_multiplier,
                &manifest.split.held_direction_indices,
            )?;
            for row in rows {
                let direction = source_manifest.fixture.listener_directions[row.direction_index];
                let cosine = normalized_cosine(direction)?;
                let wave_number = *wave_number_radius / source_manifest.fixture.radius_metres;
                let radius = source_manifest.fixture.radius_metres * *radius_multiplier;
                let basis = multipole_basis(wave_number, radius, cosine)?;
                let predicted = evaluate(&basis, &fit.coefficients)?;
                held_conditions.push(evaluate_condition(row, predicted)?);
            }
        }
    }
    let aggregate = aggregate(&held_conditions, &frequencies)?;
    let gates = GateReport {
        every_source_gate_passed: source_report.gate.values().all(|passed| *passed),
        degree_two_energy_passed: aggregate.minimum_degree_two_coefficient_energy_fraction
            >= manifest
                .admission_gates
                .minimum_degree_two_coefficient_energy_fraction,
        held_peak_normalized_error_passed: aggregate.held_max_peak_normalized_complex_error
            <= manifest
                .admission_gates
                .held_max_peak_normalized_complex_error,
        held_active_relative_error_passed: aggregate.held_max_active_relative_complex_error
            <= manifest
                .admission_gates
                .held_max_active_relative_complex_error,
        held_active_magnitude_error_passed: aggregate.held_max_active_absolute_magnitude_error_db
            <= manifest
                .admission_gates
                .held_max_active_absolute_magnitude_error_db,
        held_active_phase_error_passed: aggregate.held_max_active_absolute_phase_error_degrees
            <= manifest
                .admission_gates
                .held_max_active_absolute_phase_error_degrees,
        held_directional_correlation_passed: aggregate.held_min_directional_complex_correlation
            >= manifest
                .admission_gates
                .held_min_directional_complex_correlation,
    };
    let passed = gates.all_passed();
    Ok(Report {
        schema: REPORT_SCHEMA,
        status: "Validated",
        decision: if passed {
            "SurfaceModeOutgoingMultipoleCookerSupported"
        } else {
            "SurfaceModeOutgoingMultipoleCookerRejected"
        },
        claim: "SYNTHETIC_AXISYMMETRIC_QUADRUPOLE_NEAR_TO_FAR_COOKER_ONLY / NO_NONSPHERICAL_GEOMETRY_FEM_REAL_OBJECT_MATERIAL_QUALITY_ADMISSION_RUNTIME_OR_REALIMPACT_CREDIT",
        study_id: STUDY_ID,
        protocol_revision: PROTOCOL_REVISION,
        manifest_sha256: MANIFEST_SHA256,
        source_manifest_sha256: SOURCE_MANIFEST_SHA256,
        source_report_sha256: SOURCE_REPORT_SHA256,
        split: &manifest.split,
        candidate: &manifest.candidate,
        frequencies,
        aggregate,
        thresholds: &manifest.admission_gates,
        gate: gates,
        held_conditions,
        allowed_claims: &manifest.allowed_claims,
        prohibited_claims: &manifest.prohibited_claims,
        data_policy: &manifest.data_policy,
        next_action: if passed {
            "freeze one non-spherical closed mesh with a prescribed surface mode, require mesh/refinement agreement against Bempp, and keep REALIMPACT sealed until that geometry transfer passes"
        } else {
            "keep REALIMPACT sealed and diagnose outgoing-Hankel convention, fit conditioning or near-to-far basis sufficiency on the unchanged quadrupole oracle"
        },
    })
}

fn select_conditions<'a>(
    conditions: &'a [SourceCondition],
    wave_number_radius: f64,
    listener_radius_multiplier: f64,
    directions: &[usize],
) -> Result<Vec<&'a SourceCondition>, String> {
    directions
        .iter()
        .map(|direction| {
            conditions
                .iter()
                .find(|condition| {
                    condition.wave_number_radius == wave_number_radius
                        && condition.listener_radius_multiplier == listener_radius_multiplier
                        && condition.direction_index == *direction
                })
                .ok_or_else(|| "surface-mode cooker split condition is missing".to_owned())
        })
        .collect()
}

fn evaluate_condition(
    source: &SourceCondition,
    predicted: ComplexValue,
) -> Result<HeldCondition, String> {
    let difference = predicted.subtract(source.computed);
    let peak_error = difference.magnitude() / source.analytical_peak_magnitude;
    let active_relative_error = source
        .active_lobe
        .then(|| difference.magnitude() / source.computed.magnitude());
    let active_magnitude_error_db = source
        .active_lobe
        .then(|| (20.0 * (predicted.magnitude() / source.computed.magnitude()).log10()).abs());
    let active_phase_error_degrees = source.active_lobe.then(|| {
        let delta = predicted.phase() - source.computed.phase();
        delta.sin().atan2(delta.cos()).to_degrees().abs()
    });
    if !peak_error.is_finite()
        || active_relative_error.is_some_and(|value| !value.is_finite())
        || active_magnitude_error_db.is_some_and(|value| !value.is_finite())
        || active_phase_error_degrees.is_some_and(|value| !value.is_finite())
    {
        return Err("surface-mode cooker held metric is non-finite".to_owned());
    }
    Ok(HeldCondition {
        wave_number_radius: source.wave_number_radius,
        frequency_hz: source.frequency_hz,
        listener_radius_multiplier: source.listener_radius_multiplier,
        direction_index: source.direction_index,
        angular_profile_value: source.angular_profile_value,
        active_lobe: source.active_lobe,
        nodal_direction: source.nodal_direction,
        target: source.computed,
        predicted,
        analytical_peak_magnitude: source.analytical_peak_magnitude,
        peak_normalized_complex_error: peak_error,
        active_relative_complex_error: active_relative_error,
        active_absolute_magnitude_error_db: active_magnitude_error_db,
        active_absolute_phase_error_degrees: active_phase_error_degrees,
    })
}

fn aggregate(
    conditions: &[HeldCondition],
    frequencies: &[FrequencyReport],
) -> Result<Aggregate, String> {
    if conditions.len() != 132 || frequencies.len() != 3 {
        return Err("surface-mode cooker aggregate dimensions changed".to_owned());
    }
    let peak_errors = conditions
        .iter()
        .map(|row| row.peak_normalized_complex_error)
        .collect::<Vec<_>>();
    let active_relative = present_values(conditions, |row| row.active_relative_complex_error)?;
    let active_magnitude =
        present_values(conditions, |row| row.active_absolute_magnitude_error_db)?;
    let active_phase = present_values(conditions, |row| row.active_absolute_phase_error_degrees)?;
    let mut correlations = Vec::new();
    for wave_number_radius in [0.25, 0.75, 1.5] {
        for radius_multiplier in [3.0, 10.0] {
            let group = conditions
                .iter()
                .filter(|row| {
                    row.wave_number_radius == wave_number_radius
                        && row.listener_radius_multiplier == radius_multiplier
                })
                .collect::<Vec<_>>();
            if group.len() != 22 {
                return Err("surface-mode cooker correlation group is incomplete".to_owned());
            }
            correlations.push(DirectionalCorrelation {
                wave_number_radius,
                listener_radius_multiplier: radius_multiplier,
                complex_correlation: complex_correlation(&group)?,
            });
        }
    }
    Ok(Aggregate {
        frequency_count: frequencies.len(),
        held_condition_count: conditions.len(),
        held_active_condition_count: active_relative.len(),
        held_median_peak_normalized_complex_error: median(&peak_errors)?,
        held_max_peak_normalized_complex_error: maximum(&peak_errors)?,
        held_max_active_relative_complex_error: maximum(&active_relative)?,
        held_max_active_absolute_magnitude_error_db: maximum(&active_magnitude)?,
        held_max_active_absolute_phase_error_degrees: maximum(&active_phase)?,
        held_min_directional_complex_correlation: correlations
            .iter()
            .map(|row| row.complex_correlation)
            .min_by(f64::total_cmp)
            .ok_or_else(|| "surface-mode cooker has no correlations".to_owned())?,
        minimum_degree_two_coefficient_energy_fraction: frequencies
            .iter()
            .map(|row| row.degree_two_coefficient_energy_fraction)
            .min_by(f64::total_cmp)
            .ok_or_else(|| "surface-mode cooker has no frequency fits".to_owned())?,
        directional_correlations: correlations,
    })
}

fn present_values(
    conditions: &[HeldCondition],
    select: impl Fn(&HeldCondition) -> Option<f64>,
) -> Result<Vec<f64>, String> {
    let values = conditions.iter().filter_map(select).collect::<Vec<_>>();
    if values.is_empty() || values.iter().any(|value| !value.is_finite()) {
        return Err("surface-mode cooker has invalid optional metrics".to_owned());
    }
    Ok(values)
}

fn complex_correlation(rows: &[&HeldCondition]) -> Result<f64, String> {
    let mut numerator = ComplexValue::ZERO;
    let mut target_energy = 0.0;
    let mut predicted_energy = 0.0;
    for row in rows {
        numerator = numerator.add(row.target.conjugate().multiply(row.predicted));
        target_energy += row.target.magnitude_squared();
        predicted_energy += row.predicted.magnitude_squared();
    }
    let denominator = (target_energy * predicted_energy).sqrt();
    let value = numerator.magnitude() / denominator;
    if value.is_finite() && denominator > 1.0e-30 {
        Ok(value)
    } else {
        Err("surface-mode cooker correlation is invalid".to_owned())
    }
}

fn median(values: &[f64]) -> Result<f64, String> {
    let mut values = values.to_vec();
    if values.is_empty() || values.iter().any(|value| !value.is_finite()) {
        return Err("surface-mode cooker median input is invalid".to_owned());
    }
    values.sort_by(f64::total_cmp);
    Ok(if values.len().is_multiple_of(2) {
        (values[values.len() / 2 - 1] + values[values.len() / 2]) * 0.5
    } else {
        values[values.len() / 2]
    })
}

fn maximum(values: &[f64]) -> Result<f64, String> {
    values
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .max_by(f64::total_cmp)
        .ok_or_else(|| "surface-mode cooker maximum input is invalid".to_owned())
}

fn pretty_json(value: &impl Serialize) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("serialize surface-mode cooker report: {error}"))?;
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
        .ok_or_else(|| "surface-mode cooker output has no parent".to_owned())?;
    let sequence = NEXT_STAGING.fetch_add(1, Ordering::Relaxed);
    let staging = parent.join(format!(
        ".nextengine-surface-mode-cooker-{}-{sequence}",
        std::process::id()
    ));
    fs::create_dir(&staging)
        .map_err(|error| format!("create surface-mode cooker staging: {error}"))?;
    let result = (|| {
        fs::write(staging.join("manifest.json"), manifest)
            .map_err(|error| format!("write surface-mode cooker manifest: {error}"))?;
        fs::write(staging.join("source-manifest.json"), source_manifest)
            .map_err(|error| format!("write surface-mode source manifest: {error}"))?;
        fs::write(staging.join("source-report.json"), source_report)
            .map_err(|error| format!("write surface-mode source report: {error}"))?;
        fs::write(staging.join("report.json"), report)
            .map_err(|error| format!("write surface-mode cooker report: {error}"))?;
        if output.exists() {
            fs::remove_dir(output).map_err(|error| {
                format!("remove confirmed-empty surface-mode cooker output: {error}")
            })?;
        }
        fs::rename(&staging, output)
            .map_err(|error| format!("publish surface-mode cooker output: {error}"))
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
    source_manifest: ArtifactReference,
    source_report: ArtifactReference,
    source_contract: SourceContract,
    split: Split,
    candidate: Candidate,
    admission_gates: AdmissionGates,
    allowed_claims: Vec<String>,
    prohibited_claims: Vec<String>,
    data_policy: DataPolicy,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ArtifactReference {
    path: PathBuf,
    sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceContract {
    schema: String,
    study_id: String,
    protocol_revision: String,
    manifest_sha256: String,
    decision: String,
    mesh_refinement_level: usize,
    panel_count: usize,
    condition_count: usize,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Split {
    fit_listener_radius_multiplier: f64,
    fit_direction_indices: Vec<usize>,
    held_listener_radius_multipliers: Vec<f64>,
    held_direction_indices: Vec<usize>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Candidate {
    id: String,
    maximum_order: usize,
    ridge: f64,
    speed_of_sound_metres_per_second: f64,
    radial_basis: String,
    angular_basis: String,
    coefficient_fit: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct AdmissionGates {
    require_every_source_gate_passed: bool,
    minimum_degree_two_coefficient_energy_fraction: f64,
    held_max_peak_normalized_complex_error: f64,
    held_max_active_relative_complex_error: f64,
    held_max_active_absolute_magnitude_error_db: f64,
    held_max_active_absolute_phase_error_degrees: f64,
    held_min_directional_complex_correlation: f64,
    repeat_report_bytes_identical: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DataPolicy {
    generated_analytical_fixture_only: bool,
    fresh_realimpact_payload_access_allowed: bool,
    network_training_allowed: bool,
    runtime_or_quality_admission_credit_allowed: bool,
}

#[derive(Debug, Deserialize)]
struct SourceManifest {
    schema: String,
    study_id: String,
    protocol_revision: String,
    fixture: SourceFixture,
}

#[derive(Debug, Deserialize)]
struct SourceFixture {
    radius_metres: f64,
    speed_of_sound_metres_per_second: f64,
    spherical_harmonic_degree: usize,
    spherical_harmonic_order: usize,
    angular_profile: String,
    wave_number_radius_values: Vec<f64>,
    listener_radius_multipliers: Vec<f64>,
    listener_directions: Vec<[f64; 3]>,
}

#[derive(Debug, Deserialize)]
struct SourceReport {
    schema: String,
    study_id: String,
    protocol_revision: String,
    manifest_sha256: String,
    decision: String,
    gate: std::collections::BTreeMap<String, bool>,
    fine: SourceLevel,
}

#[derive(Debug, Deserialize)]
struct SourceLevel {
    mesh_refinement_level: usize,
    panel_count: usize,
    condition_count: usize,
    conditions: Vec<SourceCondition>,
}

#[derive(Debug, Deserialize)]
struct SourceCondition {
    wave_number_radius: f64,
    frequency_hz: f64,
    listener_radius_multiplier: f64,
    direction_index: usize,
    angular_profile_value: f64,
    active_lobe: bool,
    nodal_direction: bool,
    computed: ComplexValue,
    analytical_peak_magnitude: f64,
}

impl SourceCondition {
    fn is_finite(&self) -> bool {
        self.wave_number_radius.is_finite()
            && self.frequency_hz.is_finite()
            && self.listener_radius_multiplier.is_finite()
            && self.angular_profile_value.is_finite()
            && self.computed.is_finite()
            && self.analytical_peak_magnitude.is_finite()
            && self.analytical_peak_magnitude > 0.0
    }
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
    split: &'a Split,
    candidate: &'a Candidate,
    frequencies: Vec<FrequencyReport>,
    aggregate: Aggregate,
    thresholds: &'a AdmissionGates,
    gate: GateReport,
    held_conditions: Vec<HeldCondition>,
    allowed_claims: &'a [String],
    prohibited_claims: &'a [String],
    data_policy: &'a DataPolicy,
    next_action: &'static str,
}

#[derive(Debug, Serialize)]
struct FrequencyReport {
    wave_number_radius: f64,
    frequency_hz: f64,
    fit_condition_count: usize,
    coefficients: Vec<ComplexValue>,
    coefficient_energy_fraction_by_degree: Vec<f64>,
    degree_two_coefficient_energy_fraction: f64,
    fit_max_peak_normalized_complex_error: f64,
}

#[derive(Debug, Serialize)]
struct HeldCondition {
    wave_number_radius: f64,
    frequency_hz: f64,
    listener_radius_multiplier: f64,
    direction_index: usize,
    angular_profile_value: f64,
    active_lobe: bool,
    nodal_direction: bool,
    target: ComplexValue,
    predicted: ComplexValue,
    analytical_peak_magnitude: f64,
    peak_normalized_complex_error: f64,
    active_relative_complex_error: Option<f64>,
    active_absolute_magnitude_error_db: Option<f64>,
    active_absolute_phase_error_degrees: Option<f64>,
}

#[derive(Debug, Serialize)]
struct Aggregate {
    frequency_count: usize,
    held_condition_count: usize,
    held_active_condition_count: usize,
    held_median_peak_normalized_complex_error: f64,
    held_max_peak_normalized_complex_error: f64,
    held_max_active_relative_complex_error: f64,
    held_max_active_absolute_magnitude_error_db: f64,
    held_max_active_absolute_phase_error_degrees: f64,
    held_min_directional_complex_correlation: f64,
    minimum_degree_two_coefficient_energy_fraction: f64,
    directional_correlations: Vec<DirectionalCorrelation>,
}

#[derive(Debug, Serialize)]
struct DirectionalCorrelation {
    wave_number_radius: f64,
    listener_radius_multiplier: f64,
    complex_correlation: f64,
}

#[derive(Debug, Serialize)]
struct GateReport {
    every_source_gate_passed: bool,
    degree_two_energy_passed: bool,
    held_peak_normalized_error_passed: bool,
    held_active_relative_error_passed: bool,
    held_active_magnitude_error_passed: bool,
    held_active_phase_error_passed: bool,
    held_directional_correlation_passed: bool,
}

impl GateReport {
    fn all_passed(&self) -> bool {
        self.every_source_gate_passed
            && self.degree_two_energy_passed
            && self.held_peak_normalized_error_passed
            && self.held_active_relative_error_passed
            && self.held_active_magnitude_error_passed
            && self.held_active_phase_error_passed
            && self.held_directional_correlation_passed
    }
}
