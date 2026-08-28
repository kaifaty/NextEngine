use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;

use self::model::{
    AngularFamily, BasisColumn, ComplexValue, FitRequest, basis, evaluate, fit_coefficients,
};
use super::super::{
    canonical_external_file, read_bounded_file, require_empty_output, resolve_cli_path,
    resolve_output_path, sha256_hex,
};

mod fem;
mod model;
mod schema;

use self::schema::*;

const MANIFEST_SCHEMA: &str = "nextengine.experimental-physical-sound-triaxial-cooker.manifest.v1";
const REPORT_SCHEMA: &str = "nextengine.experimental-physical-sound-triaxial-cooker.report.v1";
const FEM_MANIFEST_SCHEMA: &str =
    "nextengine.experimental-physical-sound-fem-mode-cooker.manifest.v1";
const FEM_REPORT_SCHEMA: &str = "nextengine.experimental-physical-sound-fem-mode-cooker.report.v1";
const SPARSE_MANIFEST_SHA256: &str =
    "e3bbd547dd888cafa0be345d6abe3cb4d3c1569a2f12f362306bcb68b8c3851e";
const FULL_SHELL_MANIFEST_SHA256: &str =
    "e69f09b20b4f3bef57c8008fe114ddc5e6ba7f3220baf3657fe8d1f624d1b896";
const FEM_MANIFEST_SHA256: &str =
    "d4daf0f3fa2e421330634789613f8c40130253bf69088077b140b94291fa4cf1";
const SOURCE_MANIFEST_SHA256: &str =
    "74d8ebd1339faddf5727d0e02e1671a3a2a9295b83524f1f9336e1ec3cee267d";
const SOURCE_REPORT_SHA256: &str =
    "49bee8c27202eb89edee9be06def177f883bd751272a1700be98454bf31376ef";
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
            _ => return Err(format!("unexpected triaxial-cooker argument: {flag}")),
        }
    }
    let manifest = manifest.ok_or_else(|| {
        "physical-sound-registry bem-feasibility triaxial-cooker requires --manifest <external-json>"
            .to_owned()
    })?;
    let output = output.ok_or_else(|| {
        "physical-sound-registry bem-feasibility triaxial-cooker requires --output <external-empty-directory>"
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
        "triaxial cooker manifest",
    )?;
    let manifest_bytes = read_bounded_file(
        &manifest_path,
        MAX_MANIFEST_BYTES,
        "triaxial cooker manifest",
    )?;
    let manifest_sha256 = sha256_hex(&manifest_bytes);
    if ![
        SPARSE_MANIFEST_SHA256,
        FULL_SHELL_MANIFEST_SHA256,
        FEM_MANIFEST_SHA256,
    ]
    .contains(&manifest_sha256.as_str())
    {
        return Err(format!(
            "triaxial cooker manifest hash is not registered: {manifest_sha256}"
        ));
    }
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("parse triaxial cooker manifest: {error}"))?;
    validate_manifest(&manifest, &manifest_sha256)?;
    let base = manifest_path
        .parent()
        .ok_or_else(|| "triaxial cooker manifest has no parent".to_owned())?;
    let source_manifest_bytes = read_reference(
        root,
        base,
        &manifest.source_manifest,
        MAX_MANIFEST_BYTES,
        "triaxial Bempp source manifest",
    )?;
    let source_report_bytes = read_reference(
        root,
        base,
        &manifest.source_report,
        MAX_SOURCE_REPORT_BYTES,
        "triaxial Bempp source report",
    )?;
    let source_manifest: SourceManifest = serde_json::from_slice(&source_manifest_bytes)
        .map_err(|error| format!("parse triaxial Bempp source manifest: {error}"))?;
    let source_report: SourceReport = serde_json::from_slice(&source_report_bytes)
        .map_err(|error| format!("parse triaxial Bempp source report: {error}"))?;
    validate_source(&manifest, &source_manifest, &source_report)?;

    let report = build_report(
        &manifest,
        &manifest_sha256,
        &source_manifest,
        &source_report,
    )?;
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

    println!("physical sound triaxial cooker: {}", output.display());
    println!("manifest sha256: {manifest_sha256}");
    for candidate in &report.candidates {
        println!(
            "candidate {}: peak {:.9}, correlation {:.9}, passed {}",
            candidate.id,
            candidate.aggregate.held_max_peak_normalized_complex_error,
            candidate.aggregate.held_min_directional_complex_correlation,
            candidate.gate.all_passed()
        );
    }
    println!(
        "selected candidate: {}",
        report.selected_candidate_id.as_deref().unwrap_or("none")
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

fn validate_manifest(manifest: &Manifest, manifest_sha256: &str) -> Result<(), String> {
    if manifest_sha256 == FEM_MANIFEST_SHA256 {
        return fem::validate_manifest(manifest);
    }
    let sparse = manifest_sha256 == SPARSE_MANIFEST_SHA256;
    let expected_fit = if sparse {
        (0..16).chain(24..32).chain(40..56).collect::<Vec<_>>()
    } else {
        (0..56).collect::<Vec<_>>()
    };
    let expected_near = if sparse {
        (16..24).chain(32..40).collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let expected_far = (0..56).collect::<Vec<_>>();
    let mut expected_candidates = vec![
        (
            "axisymmetric-mzero-outgoing-order4-ridge1e12-v1",
            AngularFamily::AxisymmetricMZero,
            4,
        ),
        (
            "full-real-spherical-angular-outgoing-order2-ridge1e12-v1",
            AngularFamily::FullRealSphericalHarmonics,
            2,
        ),
        (
            "full-real-spherical-angular-outgoing-order4-ridge1e12-v1",
            AngularFamily::FullRealSphericalHarmonics,
            4,
        ),
    ];
    if !sparse {
        expected_candidates.push((
            "full-real-spherical-angular-outgoing-order6-ridge1e12-v1",
            AngularFamily::FullRealSphericalHarmonics,
            6,
        ));
    }
    let candidates_match = manifest.candidates.len() == expected_candidates.len()
        && manifest
            .candidates
            .iter()
            .zip(expected_candidates)
            .all(|(actual, expected)| {
                actual.id == expected.0
                    && actual.angular_family == expected.1
                    && actual.maximum_degree == expected.2
                    && actual.ridge == 1.0e-12
            });
    let identity_matches = if sparse {
        manifest.study_id == "physical-sound-full-angular-cooker-triaxial-control"
            && manifest.protocol_revision == "outgoing-spherical-angular-candidates-v1"
            && manifest.allowed_claims
                == [
                    "synthetic_triaxial_nonaxisymmetric_surface_mode_cooker_sufficiency",
                    "near_to_far_full_angular_outgoing_multipole_transfer",
                ]
    } else {
        manifest.study_id == "physical-sound-full-near-shell-cooker-triaxial-control"
            && manifest.protocol_revision == "outgoing-spherical-full-near-shell-candidates-v1"
            && manifest.allowed_claims
                == [
                    "synthetic_triaxial_full_near_shell_cooker_sufficiency",
                    "near_to_far_full_angular_outgoing_multipole_transfer",
                ]
    };
    if manifest.schema != MANIFEST_SCHEMA
        || !identity_matches
        || manifest.source_manifest.path != Path::new("run-a/manifest.json")
        || manifest.source_manifest.sha256 != SOURCE_MANIFEST_SHA256
        || manifest.source_report.path != Path::new("run-a/report.json")
        || manifest.source_report.sha256 != SOURCE_REPORT_SHA256
        || manifest.source_contract.schema
            != "nextengine.experimental-physical-sound-bempp-triaxial-mode.report.v1"
        || manifest.source_contract.study_id != "physical-sound-independent-bempp-triaxial-xz-mode"
        || manifest.source_contract.protocol_revision
            != "bempp-direct-neumann-to-dirichlet-triaxial-xz-v1"
        || manifest.source_contract.manifest_sha256 != SOURCE_MANIFEST_SHA256
        || manifest.source_contract.decision != "IndependentBemppTriaxialSurfaceModeSupported"
        || manifest.source_contract.fine_mesh_refinement_level != 4
        || manifest.source_contract.fine_panel_count != 2048
        || manifest.source_contract.fine_condition_count != 336
        || manifest.split.fit_listener_radius_reference_multiplier != 2.0
        || manifest.split.fit_direction_indices != expected_fit
        || manifest.split.held_near_direction_indices != expected_near
        || manifest
            .split
            .held_far_listener_radius_reference_multipliers
            != [4.0, 10.0]
        || manifest.split.held_far_direction_indices != expected_far
        || !candidates_match
        || manifest.shared_model.speed_of_sound_metres_per_second != 343.0
        || manifest.shared_model.radial_basis != "spherical-hankel-first-kind"
        || manifest.shared_model.angular_basis != "associated-legendre-times-real-sine-cosine"
        || manifest.shared_model.coefficient_fit
            != "complex-column-scaled-ridge-normal-equations-partial-pivot"
        || manifest.shared_model.active_field_minimum_peak_ratio != 0.25
        || manifest
            .shared_model
            .field_classification_absolute_tolerance
            != 1.0e-12
        || !manifest.admission_gates.require_every_source_gate_passed
        || !manifest
            .admission_gates
            .require_axisymmetric_control_rejected
        || manifest.admission_gates.selection_rule
            != "smallest-full-angular-maximum-degree-passing-all-held-gates"
        || !manifest.admission_gates.repeat_report_bytes_identical
        || manifest.prohibited_claims.len() != 6
        || !manifest.data_policy.generated_synthetic_fixture_only
        || manifest.data_policy.fresh_realimpact_payload_access_allowed
        || manifest.data_policy.network_training_allowed
        || manifest
            .data_policy
            .runtime_or_quality_admission_credit_allowed
    {
        return Err("triaxial cooker manifest does not match the frozen protocol".to_owned());
    }
    Ok(())
}

fn validate_source(
    manifest: &Manifest,
    source_manifest: &SourceManifest,
    source_report: &SourceReport,
) -> Result<(), String> {
    if manifest.schema == FEM_MANIFEST_SCHEMA {
        return fem::validate_source(manifest, source_manifest, source_report);
    }
    let fixture = &source_manifest.fixture;
    let contract = &manifest.source_contract;
    let fine = source_report
        .fine
        .as_ref()
        .ok_or_else(|| "triaxial Bempp source has no fine level".to_owned())?;
    if source_manifest.schema
        != "nextengine.experimental-physical-sound-bempp-triaxial-mode.manifest.v1"
        || source_manifest.study_id != contract.study_id
        || source_manifest.protocol_revision != contract.protocol_revision
        || fixture.id != "unit-normal-derivative-triaxial-ellipsoid-xz-mode"
        || fixture.reference_length_metres != 0.1
        || fixture.semiaxes_metres != [0.08, 0.1, 0.13]
        || fixture.speed_of_sound_metres_per_second != 343.0
        || fixture.surface_mode != "two-times-parametric-unit-x-times-z"
        || fixture.surface_profile_projection != "ellipsoid-radial-pullback"
        || fixture.wave_number_reference_length_values != [0.75, 1.5]
        || fixture.listener_radius_reference_multipliers != [2.0, 4.0, 10.0]
        || fixture.listener_directions.len() != 56
        || source_report.schema != contract.schema
        || source_report.study_id != contract.study_id
        || source_report.protocol_revision != contract.protocol_revision
        || source_report.manifest_sha256 != contract.manifest_sha256
        || source_report.decision != contract.decision
        || fine.mesh_refinement_level != Some(contract.fine_mesh_refinement_level)
        || fine.panel_count != contract.fine_panel_count
        || fine.condition_count != contract.fine_condition_count
        || fine.conditions.len() != contract.fine_condition_count
        || source_report.gate.len() != 9
        || source_report.gate.values().any(|passed| !passed)
    {
        return Err("triaxial Bempp source does not match the frozen contract".to_owned());
    }
    for direction in &fixture.listener_directions {
        let norm = direction
            .iter()
            .map(|value| value * value)
            .sum::<f64>()
            .sqrt();
        if direction.iter().any(|value| !value.is_finite()) || (norm - 1.0).abs() > 1.0e-12 {
            return Err("triaxial Bempp listener direction is invalid".to_owned());
        }
    }
    for condition in &fine.conditions {
        if !condition.is_finite()
            || !fixture
                .wave_number_reference_length_values
                .contains(&condition.wave_number_reference_length)
            || !fixture
                .listener_radius_reference_multipliers
                .contains(&condition.listener_radius_reference_multiplier)
            || condition.direction_index >= fixture.listener_directions.len()
        {
            return Err("triaxial Bempp source condition is invalid".to_owned());
        }
    }
    for wave_number in &fixture.wave_number_reference_length_values {
        for radius in &fixture.listener_radius_reference_multipliers {
            for direction_index in 0..fixture.listener_directions.len() {
                let count = fine
                    .conditions
                    .iter()
                    .filter(|condition| {
                        condition.wave_number_reference_length == *wave_number
                            && condition.listener_radius_reference_multiplier == *radius
                            && condition.direction_index == direction_index
                    })
                    .count();
                if count != 1 {
                    return Err("triaxial Bempp source condition grid is incomplete".to_owned());
                }
            }
        }
    }
    Ok(())
}

fn build_report<'a>(
    manifest: &'a Manifest,
    manifest_sha256: &'a str,
    source_manifest: &SourceManifest,
    source_report: &SourceReport,
) -> Result<Report<'a>, String> {
    let fem_mode = manifest.schema == FEM_MANIFEST_SCHEMA;
    let mut candidates = Vec::new();
    for candidate in &manifest.candidates {
        candidates.push(evaluate_candidate(
            manifest,
            candidate,
            source_manifest,
            source_report,
        )?);
    }
    let axisymmetric_control_rejected = candidates
        .first()
        .is_some_and(|candidate| !candidate.gate.all_passed());
    let selected_candidate_id = candidates
        .iter()
        .filter(|report| {
            report.angular_family == AngularFamily::FullRealSphericalHarmonics
                && report.gate.all_passed()
        })
        .min_by_key(|report| report.maximum_degree)
        .map(|report| report.id.clone());
    let gate = OverallGate {
        every_source_gate_passed: source_report.gate.values().all(|passed| *passed),
        axisymmetric_control_rejected,
        full_angular_candidate_selected: selected_candidate_id.is_some(),
    };
    let passed = gate.all_passed();
    Ok(Report {
        schema: if fem_mode {
            FEM_REPORT_SCHEMA
        } else {
            REPORT_SCHEMA
        },
        status: "Validated",
        decision: if fem_mode && passed {
            "FullAngularElasticFemModeNearToFarCookerSupported"
        } else if fem_mode {
            "FullAngularElasticFemModeNearToFarCookerRejected"
        } else if passed {
            "FullAngularTriaxialNearToFarCookerSupported"
        } else {
            "FullAngularTriaxialNearToFarCookerRejected"
        },
        claim: if fem_mode {
            "SYNTHETIC_ELASTIC_FEM_MODE_NEAR_TO_FAR_COOKER_ONLY / NO_REAL_OBJECT_MATERIAL_QUALITY_ADMISSION_RUNTIME_OR_REALIMPACT_CREDIT"
        } else {
            "SYNTHETIC_TRIAXIAL_NONAXISYMMETRIC_NEAR_TO_FAR_COOKER_ONLY / NO_FEM_REAL_OBJECT_MATERIAL_QUALITY_ADMISSION_RUNTIME_OR_REALIMPACT_CREDIT"
        },
        study_id: &manifest.study_id,
        protocol_revision: &manifest.protocol_revision,
        manifest_sha256,
        source_manifest_sha256: &manifest.source_manifest.sha256,
        source_report_sha256: &manifest.source_report.sha256,
        split: &manifest.split,
        candidates,
        selected_candidate_id,
        thresholds: &manifest.admission_gates,
        gate,
        allowed_claims: &manifest.allowed_claims,
        prohibited_claims: &manifest.prohibited_claims,
        data_policy: &manifest.data_policy,
        next_action: if fem_mode && passed {
            "preregister a fresh object-disjoint real-data spatial-transfer calibration while keeping its payload sealed until the protocol is immutable"
        } else if fem_mode {
            "keep REALIMPACT sealed and diagnose FEM surface-mode convergence, angular order or fit conditioning against the unchanged synthetic controls"
        } else if passed {
            "couple one actual synthetic FEM surface eigenvector on a non-spherical closed mesh to the same independent Bempp and full-angular cooker path before any fresh REALIMPACT payload"
        } else {
            "keep REALIMPACT sealed and diagnose angular order, spherical expansion origin or fit conditioning against the unchanged triaxial oracle"
        },
    })
}

fn evaluate_candidate(
    manifest: &Manifest,
    candidate: &Candidate,
    source_manifest: &SourceManifest,
    source_report: &SourceReport,
) -> Result<CandidateReport, String> {
    let fixture = &source_manifest.fixture;
    let source_level = source_level(source_report)?;
    let directions = source_directions(source_manifest, source_report)?;
    let wave_numbers = source_wave_numbers(source_manifest, source_report)?;
    let mut frequencies = Vec::new();
    let mut held_conditions = Vec::new();
    for wave_number in &wave_numbers {
        let fit_rows = select_conditions(
            &source_level.conditions,
            *wave_number,
            manifest.split.fit_listener_radius_reference_multiplier,
            &manifest.split.fit_direction_indices,
        )?;
        let fit = fit_coefficients(&FitRequest {
            rows: &fit_rows,
            directions,
            wave_number_reference_length: *wave_number,
            reference_length_metres: fixture.reference_length_metres,
            family: candidate.angular_family,
            maximum_degree: candidate.maximum_degree,
            ridge: candidate.ridge,
        })?;
        frequencies.push(frequency_report(*wave_number, &fit_rows, &fit)?);
        if !manifest.split.held_near_direction_indices.is_empty() {
            append_held_group(
                &mut held_conditions,
                HeldGroupRequest {
                    conditions: &source_level.conditions,
                    directions,
                    direction_indices: &manifest.split.held_near_direction_indices,
                    wave_number_reference_length: *wave_number,
                    radius_multiplier: manifest.split.fit_listener_radius_reference_multiplier,
                    reference_length_metres: fixture.reference_length_metres,
                    columns: &fit.columns,
                    coefficients: &fit.coefficients,
                    active_minimum_peak_ratio: manifest
                        .shared_model
                        .active_field_minimum_peak_ratio,
                    classification_tolerance: manifest
                        .shared_model
                        .field_classification_absolute_tolerance,
                },
            )?;
        }
        for radius in &manifest
            .split
            .held_far_listener_radius_reference_multipliers
        {
            append_held_group(
                &mut held_conditions,
                HeldGroupRequest {
                    conditions: &source_level.conditions,
                    directions,
                    direction_indices: &manifest.split.held_far_direction_indices,
                    wave_number_reference_length: *wave_number,
                    radius_multiplier: *radius,
                    reference_length_metres: fixture.reference_length_metres,
                    columns: &fit.columns,
                    coefficients: &fit.coefficients,
                    active_minimum_peak_ratio: manifest
                        .shared_model
                        .active_field_minimum_peak_ratio,
                    classification_tolerance: manifest
                        .shared_model
                        .field_classification_absolute_tolerance,
                },
            )?;
        }
    }
    let aggregate = aggregate(&held_conditions, wave_numbers.len())?;
    let gates = &manifest.admission_gates;
    let gate = CandidateGate {
        held_peak_normalized_error_passed: aggregate.held_max_peak_normalized_complex_error
            <= gates.held_max_peak_normalized_complex_error,
        held_active_relative_error_passed: aggregate.held_max_active_relative_complex_error
            <= gates.held_max_active_relative_complex_error,
        held_active_magnitude_error_passed: aggregate.held_max_active_absolute_magnitude_error_db
            <= gates.held_max_active_absolute_magnitude_error_db,
        held_active_phase_error_passed: aggregate.held_max_active_absolute_phase_error_degrees
            <= gates.held_max_active_absolute_phase_error_degrees,
        held_directional_correlation_passed: aggregate.held_min_directional_complex_correlation
            >= gates.held_min_directional_complex_correlation,
    };
    Ok(CandidateReport {
        id: candidate.id.clone(),
        angular_family: candidate.angular_family,
        maximum_degree: candidate.maximum_degree,
        frequencies,
        aggregate,
        gate,
        held_conditions,
    })
}

fn source_level(source_report: &SourceReport) -> Result<&SourceLevel, String> {
    if source_report.schema == "nextengine.experimental-physical-sound-bempp-fem-mode.report.v1" {
        source_report
            .bem_fine
            .as_ref()
            .ok_or_else(|| "FEM/Bempp source has no fine BEM level".to_owned())
    } else {
        source_report
            .fine
            .as_ref()
            .ok_or_else(|| "triaxial Bempp source has no fine level".to_owned())
    }
}

fn source_directions<'a>(
    source_manifest: &'a SourceManifest,
    source_report: &'a SourceReport,
) -> Result<&'a [[f64; 3]], String> {
    let directions = if source_report.schema
        == "nextengine.experimental-physical-sound-bempp-fem-mode.report.v1"
    {
        &source_report.listener_directions
    } else {
        &source_manifest.fixture.listener_directions
    };
    if directions.is_empty() {
        Err("triaxial cooker source has no listener directions".to_owned())
    } else {
        Ok(directions)
    }
}

fn source_wave_numbers(
    source_manifest: &SourceManifest,
    source_report: &SourceReport,
) -> Result<Vec<f64>, String> {
    if source_report.schema == "nextengine.experimental-physical-sound-bempp-fem-mode.report.v1" {
        source_report
            .common_wave_number_reference_length
            .map(|value| vec![value])
            .ok_or_else(|| "FEM/Bempp source has no common wave number".to_owned())
    } else if source_manifest
        .fixture
        .wave_number_reference_length_values
        .is_empty()
    {
        Err("triaxial cooker source has no wave numbers".to_owned())
    } else {
        Ok(source_manifest
            .fixture
            .wave_number_reference_length_values
            .clone())
    }
}

fn frequency_report(
    wave_number: f64,
    rows: &[&SourceCondition],
    fit: &model::FitResult,
) -> Result<FrequencyReport, String> {
    let maximum_degree = fit
        .columns
        .iter()
        .map(|column| column.degree)
        .max()
        .ok_or_else(|| "triaxial cooker fit has no columns".to_owned())?;
    let total_energy = fit
        .coefficients
        .iter()
        .map(|coefficient| coefficient.magnitude_squared())
        .sum::<f64>();
    if !total_energy.is_finite() {
        return Err("triaxial cooker coefficient energy is invalid".to_owned());
    }
    let energy_by_degree = if total_energy > 1.0e-30 {
        (0..=maximum_degree)
            .map(|degree| {
                fit.columns
                    .iter()
                    .zip(&fit.coefficients)
                    .filter(|(column, _)| column.degree == degree)
                    .map(|(_, coefficient)| coefficient.magnitude_squared())
                    .sum::<f64>()
                    / total_energy
            })
            .collect()
    } else {
        vec![0.0; maximum_degree + 1]
    };
    Ok(FrequencyReport {
        wave_number_reference_length: wave_number,
        frequency_hz: rows[0].frequency_hz,
        fit_condition_count: rows.len(),
        columns: fit.columns.clone(),
        coefficients: fit.coefficients.clone(),
        coefficient_total_energy: total_energy,
        coefficient_energy_fraction_by_degree: energy_by_degree,
        fit_max_peak_normalized_complex_error: fit.max_peak_normalized_complex_error,
    })
}

struct HeldGroupRequest<'a> {
    conditions: &'a [SourceCondition],
    directions: &'a [[f64; 3]],
    direction_indices: &'a [usize],
    wave_number_reference_length: f64,
    radius_multiplier: f64,
    reference_length_metres: f64,
    columns: &'a [BasisColumn],
    coefficients: &'a [ComplexValue],
    active_minimum_peak_ratio: f64,
    classification_tolerance: f64,
}

fn append_held_group(
    output: &mut Vec<HeldCondition>,
    request: HeldGroupRequest<'_>,
) -> Result<(), String> {
    let rows = select_conditions(
        request.conditions,
        request.wave_number_reference_length,
        request.radius_multiplier,
        request.direction_indices,
    )?;
    let peak = rows
        .iter()
        .map(|row| row.computed.magnitude())
        .max_by(f64::total_cmp)
        .ok_or_else(|| "triaxial cooker held group is empty".to_owned())?;
    if !peak.is_finite() || peak <= 1.0e-30 {
        return Err("triaxial cooker held group peak is invalid".to_owned());
    }
    let wave_number = request.wave_number_reference_length / request.reference_length_metres;
    let radius = request.reference_length_metres * request.radius_multiplier;
    for row in rows {
        let field_ratio = row.computed.magnitude() / peak;
        let active =
            field_ratio + request.classification_tolerance >= request.active_minimum_peak_ratio;
        let basis = basis(
            wave_number,
            radius,
            request.directions[row.direction_index],
            request.columns,
        )?;
        let predicted = evaluate(&basis, request.coefficients)?;
        output.push(evaluate_condition(row, predicted, peak, active)?);
    }
    Ok(())
}

fn select_conditions<'a>(
    conditions: &'a [SourceCondition],
    wave_number: f64,
    radius: f64,
    directions: &[usize],
) -> Result<Vec<&'a SourceCondition>, String> {
    directions
        .iter()
        .map(|direction| {
            conditions
                .iter()
                .find(|condition| {
                    condition.wave_number_reference_length == wave_number
                        && condition.listener_radius_reference_multiplier == radius
                        && condition.direction_index == *direction
                })
                .ok_or_else(|| "triaxial cooker split condition is missing".to_owned())
        })
        .collect()
}

fn evaluate_condition(
    source: &SourceCondition,
    predicted: ComplexValue,
    peak: f64,
    active: bool,
) -> Result<HeldCondition, String> {
    let difference = predicted.subtract(source.computed);
    let peak_error = difference.magnitude() / peak;
    let active_relative_error =
        active.then(|| difference.magnitude() / source.computed.magnitude());
    let active_magnitude_error_db = active
        .then(|| (20.0 * (predicted.magnitude() / source.computed.magnitude()).log10()).abs());
    let active_phase_error_degrees = active.then(|| {
        let delta = predicted.phase() - source.computed.phase();
        delta.sin().atan2(delta.cos()).to_degrees().abs()
    });
    if !peak_error.is_finite()
        || active_relative_error.is_some_and(|value| !value.is_finite())
        || active_magnitude_error_db.is_some_and(|value| !value.is_finite())
        || active_phase_error_degrees.is_some_and(|value| !value.is_finite())
    {
        return Err("triaxial cooker held metric is non-finite".to_owned());
    }
    Ok(HeldCondition {
        wave_number_reference_length: source.wave_number_reference_length,
        frequency_hz: source.frequency_hz,
        listener_radius_reference_multiplier: source.listener_radius_reference_multiplier,
        direction_index: source.direction_index,
        surface_profile_direction_value: source.surface_profile_direction_value,
        active_field: active,
        target: source.computed,
        predicted,
        group_peak_magnitude: peak,
        peak_normalized_complex_error: peak_error,
        active_relative_complex_error: active_relative_error,
        active_absolute_magnitude_error_db: active_magnitude_error_db,
        active_absolute_phase_error_degrees: active_phase_error_degrees,
    })
}

fn aggregate(conditions: &[HeldCondition], frequency_count: usize) -> Result<Aggregate, String> {
    if conditions.is_empty() || frequency_count == 0 {
        return Err("triaxial cooker aggregate dimensions changed".to_owned());
    }
    let peak_errors = conditions
        .iter()
        .map(|row| row.peak_normalized_complex_error)
        .collect::<Vec<_>>();
    let active_relative = present_values(conditions, |row| row.active_relative_complex_error)?;
    let active_magnitude =
        present_values(conditions, |row| row.active_absolute_magnitude_error_db)?;
    let active_phase = present_values(conditions, |row| row.active_absolute_phase_error_degrees)?;
    let mut group_keys = Vec::new();
    for condition in conditions {
        let key = (
            condition.wave_number_reference_length,
            condition.listener_radius_reference_multiplier,
        );
        if !group_keys.contains(&key) {
            group_keys.push(key);
        }
    }
    let mut correlations = Vec::new();
    for (wave_number, radius) in group_keys {
        let group = conditions
            .iter()
            .filter(|row| {
                row.wave_number_reference_length == wave_number
                    && row.listener_radius_reference_multiplier == radius
            })
            .collect::<Vec<_>>();
        correlations.push(DirectionalCorrelation {
            wave_number_reference_length: wave_number,
            listener_radius_reference_multiplier: radius,
            direction_count: group.len(),
            complex_correlation: complex_correlation(&group)?,
        });
    }
    Ok(Aggregate {
        frequency_count,
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
            .ok_or_else(|| "triaxial cooker has no correlations".to_owned())?,
        directional_correlations: correlations,
    })
}

fn present_values(
    conditions: &[HeldCondition],
    select: impl Fn(&HeldCondition) -> Option<f64>,
) -> Result<Vec<f64>, String> {
    let values = conditions.iter().filter_map(select).collect::<Vec<_>>();
    if values.is_empty() || values.iter().any(|value| !value.is_finite()) {
        return Err("triaxial cooker has invalid optional metrics".to_owned());
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
        Err("triaxial cooker correlation is invalid".to_owned())
    }
}

fn median(values: &[f64]) -> Result<f64, String> {
    let mut values = values.to_vec();
    if values.is_empty() || values.iter().any(|value| !value.is_finite()) {
        return Err("triaxial cooker median input is invalid".to_owned());
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
        .ok_or_else(|| "triaxial cooker maximum input is invalid".to_owned())
}

fn pretty_json(value: &impl Serialize) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("serialize triaxial cooker report: {error}"))?;
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
        .ok_or_else(|| "triaxial cooker output has no parent".to_owned())?;
    let sequence = NEXT_STAGING.fetch_add(1, Ordering::Relaxed);
    let staging = parent.join(format!(
        ".nextengine-triaxial-cooker-{}-{sequence}",
        std::process::id()
    ));
    fs::create_dir(&staging).map_err(|error| format!("create triaxial cooker staging: {error}"))?;
    let result = (|| {
        fs::write(staging.join("manifest.json"), manifest)
            .map_err(|error| format!("write triaxial cooker manifest: {error}"))?;
        fs::write(staging.join("source-manifest.json"), source_manifest)
            .map_err(|error| format!("write triaxial source manifest: {error}"))?;
        fs::write(staging.join("source-report.json"), source_report)
            .map_err(|error| format!("write triaxial source report: {error}"))?;
        fs::write(staging.join("report.json"), report)
            .map_err(|error| format!("write triaxial cooker report: {error}"))?;
        if output.exists() {
            fs::remove_dir(output)
                .map_err(|error| format!("remove confirmed-empty triaxial output: {error}"))?;
        }
        fs::rename(&staging, output)
            .map_err(|error| format!("publish triaxial cooker output: {error}"))
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}
