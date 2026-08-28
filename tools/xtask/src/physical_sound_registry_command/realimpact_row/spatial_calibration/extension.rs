use std::collections::BTreeMap;
use std::f64::consts::PI;
use std::io::Read;
use std::path::{Path, PathBuf};

use flate2::read::DeflateDecoder;
use serde::{Deserialize, Serialize};

use super::super::profiles::{GLASS_GOBLET_PROFILE_ID, GREEN_GOBLET_PROFILE_ID};
use super::*;

mod publish;

const MANIFEST_SCHEMA: &str = "nextengine.experimental-realimpact-spatial-extension.manifest.v1";
const REPORT_SCHEMA: &str = "nextengine.experimental-realimpact-spatial-axis-development.report.v1";
const STUDY_ID: &str = "physical-sound-realimpact-spatial-axis-extension";
const DEVELOPMENT_REVISION: &str = "v1-development";
const DEVELOPMENT_MANIFEST_SHA256: &str =
    "c90194c809dd55afc7ab54e9fe94e09f6527f36ba5d21af081ebf03c484645fd";
const EVALUATION_REVISION: &str = "v1-evaluation";
const EVALUATION_MANIFEST_SHA256: &str =
    "dbc958bd4aacec9ff5359fe434bfa6ab0fd680c8f0aa6048e98286aec7296912";
const DEVELOPMENT_REPORT_SHA256: &str =
    "5fdaacd1a93c5af13d85ffe966889be482a544cf062306be0db58c70def3a1ed";
const SPATIAL_REPORT_SHA256: &str =
    "abc13a9cbdcaac6118197267f55264a03088130c362758e1212960dbce57389d";
const PREFIX_BYTES: u64 = 128 * 1024 * 1024;
const FIXED_CANDIDATE_ID: &str = "vertical-rbf-sigma052-ridge001-v1";
const SELECTED_BLOCK_FILE: &str = "green-axis-selected-blocks.f32le";
const ALLOWED_CLAIMS: [&str; 1] = ["development_axis_stratification_only"];
const PROHIBITED_CLAIMS: [&str; 9] = [
    "absolute_amplitude_claim",
    "angle_interpolation",
    "distance_interpolation",
    "material_identity",
    "object_generalization",
    "physical_sound_pass",
    "production_corpus_admission",
    "runtime_content_role",
    "three_dimensional_spatial_field",
];
const EVALUATION_ALLOWED_CLAIMS: [&str; 2] = [
    "fixed_candidate_two_object_vertical_condition_robustness",
    "distance_angle_stratified_holdout_result",
];
const EVALUATION_PROHIBITED_CLAIMS: [&str; 8] = [
    "absolute_amplitude_claim",
    "angle_interpolation",
    "distance_interpolation",
    "material_identity",
    "physical_sound_pass",
    "production_corpus_admission",
    "runtime_content_role",
    "three_dimensional_spatial_field",
];
const EVALUATION_PRIOR_EXPOSURES: [&str; 2] = [
    "Blue Bowl row 0 was previously used by transfer calibration V1 only; rows 1..134 remained unopened before this manifest",
    "Glass Goblet row 0 was previously used by transfer calibration V1/V2 development only; rows 1..134 remained unopened before this manifest",
];

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
            _ => return Err(format!("unexpected spatial-extension argument: {flag}")),
        }
    }
    let manifest = manifest.ok_or_else(|| {
        "physical-sound-registry spatial-extension requires --manifest <external-json>".to_owned()
    })?;
    let output = output.ok_or_else(|| {
        "physical-sound-registry spatial-extension requires --output <external-empty-directory>"
            .to_owned()
    })?;
    match requested_phase(root, &manifest)?.as_str() {
        "development" => run_development(root, &manifest, &output),
        "evaluation" => run_evaluation(root, &manifest, &output),
        phase => Err(format!("unsupported spatial-extension phase: {phase}")),
    }
}

fn requested_phase(root: &Path, manifest_argument: &Path) -> Result<String, String> {
    let manifest_path = canonical_external_file(
        root,
        &resolve_cli_path(root, manifest_argument),
        "REALIMPACT spatial-extension manifest",
    )?;
    let bytes = read_bounded_file(
        &manifest_path,
        MAX_MANIFEST_BYTES,
        "REALIMPACT spatial-extension manifest",
    )?;
    let manifest: PhaseHeader = serde_json::from_slice(&bytes)
        .map_err(|error| format!("parse spatial-extension phase: {error}"))?;
    Ok(manifest.phase)
}

fn run_development(
    root: &Path,
    manifest_argument: &Path,
    output_argument: &Path,
) -> Result<(), String> {
    let manifest_path = canonical_external_file(
        root,
        &resolve_cli_path(root, manifest_argument),
        "REALIMPACT spatial-extension manifest",
    )?;
    let manifest_bytes = read_bounded_file(
        &manifest_path,
        MAX_MANIFEST_BYTES,
        "REALIMPACT spatial-extension manifest",
    )?;
    require_hash(
        &manifest_bytes,
        DEVELOPMENT_MANIFEST_SHA256,
        "spatial-extension development manifest",
    )?;
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("parse spatial-extension manifest: {error}"))?;
    validate_development_manifest(&manifest)?;
    let base = manifest_path
        .parent()
        .ok_or_else(|| "spatial-extension manifest has no parent".to_owned())?;

    let spatial_bytes = read_ref(root, base, &manifest.spatial_calibration_report)?;
    require_hash(&spatial_bytes, SPATIAL_REPORT_SHA256, "spatial report")?;
    validate_spatial_report(&spatial_bytes)?;
    let transfer_bytes = read_ref(root, base, &manifest.transfer_calibration_report)?;
    require_hash(
        &transfer_bytes,
        TRANSFER_REPORT_SHA256,
        "transfer-calibration report",
    )?;
    let transfer: TransferReport = serde_json::from_slice(&transfer_bytes)
        .map_err(|error| format!("parse transfer-calibration report: {error}"))?;
    validate_transfer(&transfer)?;
    let modes = modes_for(&transfer, GREEN_GOBLET_PROFILE_ID)?;
    let candidate = candidate_by_id(FIXED_CANDIDATE_ID)?;

    let profile = frozen_profile(GREEN_GOBLET_PROFILE_ID)?;
    let acquisition = acquire_selected_conditions(profile, &manifest.conditions)?;
    let condition_reports = acquisition
        .conditions
        .iter()
        .map(|block| {
            let evaluation = dsp::evaluate(&block.rows, &modes, SAMPLE_RATE_HZ, candidate)?;
            Ok(ConditionReport {
                condition: block.condition.clone(),
                gate: gate(&evaluation),
                evaluation,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let aggregate = aggregate(&condition_reports)?;
    let passed_condition_count = aggregate.passed_condition_count;
    let condition_count = aggregate.condition_count;
    let report = DevelopmentReport {
        schema: REPORT_SCHEMA,
        status: "Validated",
        decision: "DevelopmentAxisStratificationMeasured",
        claim: "GREEN_GOBLET_DEVELOPMENT_AXIS_STRATIFICATION_ONLY / NO_OBJECT_ANGLE_DISTANCE_OR_3D_GENERALIZATION",
        study_id: STUDY_ID,
        revision: DEVELOPMENT_REVISION,
        manifest_sha256: DEVELOPMENT_MANIFEST_SHA256,
        spatial_calibration_report_sha256: SPATIAL_REPORT_SHA256,
        transfer_calibration_report_sha256: TRANSFER_REPORT_SHA256,
        fixed_candidate_id: FIXED_CANDIDATE_ID,
        object_profile: GREEN_GOBLET_PROFILE_ID,
        condition_reports,
        aggregate,
        acquisition: acquisition.summary(),
        allowed_claims: &ALLOWED_CLAIMS,
        prohibited_claims: &PROHIBITED_CLAIMS,
        next_action: "freeze an evaluation rule from Green only, then evaluate unchanged RBF once on unopened Blue Bowl and Glass Goblet rows",
    };
    let report_bytes = pretty_json(&report)?;
    let report_sha256 = sha256_hex(&report_bytes);
    let output = resolve_output_path(root, output_argument)?;
    require_empty_output(&output)?;
    publish::development(
        &output,
        &manifest_bytes,
        &spatial_bytes,
        &transfer_bytes,
        &acquisition.payload,
        &report_bytes,
    )?;
    println!("REALIMPACT spatial axis development: {}", output.display());
    println!("development manifest sha256: {DEVELOPMENT_MANIFEST_SHA256}");
    println!("selected block sha256: {}", acquisition.payload_sha256);
    println!(
        "condition gates passed: {}/{}",
        passed_condition_count, condition_count
    );
    println!("report sha256: {report_sha256}");
    Ok(())
}

fn run_evaluation(
    root: &Path,
    manifest_argument: &Path,
    output_argument: &Path,
) -> Result<(), String> {
    let manifest_path = canonical_external_file(
        root,
        &resolve_cli_path(root, manifest_argument),
        "REALIMPACT spatial-extension evaluation manifest",
    )?;
    let manifest_bytes = read_bounded_file(
        &manifest_path,
        MAX_MANIFEST_BYTES,
        "REALIMPACT spatial-extension evaluation manifest",
    )?;
    require_hash(
        &manifest_bytes,
        EVALUATION_MANIFEST_SHA256,
        "spatial-extension evaluation manifest",
    )?;
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("parse spatial-extension evaluation manifest: {error}"))?;
    validate_evaluation_manifest(&manifest)?;
    let base = manifest_path
        .parent()
        .ok_or_else(|| "spatial-extension evaluation manifest has no parent".to_owned())?;
    let spatial_bytes = read_ref(root, base, &manifest.spatial_calibration_report)?;
    require_hash(&spatial_bytes, SPATIAL_REPORT_SHA256, "spatial report")?;
    validate_spatial_report(&spatial_bytes)?;
    let transfer_bytes = read_ref(root, base, &manifest.transfer_calibration_report)?;
    require_hash(
        &transfer_bytes,
        TRANSFER_REPORT_SHA256,
        "transfer-calibration report",
    )?;
    let transfer: TransferReport = serde_json::from_slice(&transfer_bytes)
        .map_err(|error| format!("parse transfer-calibration report: {error}"))?;
    validate_transfer(&transfer)?;
    let development_ref = manifest
        .development_report
        .as_ref()
        .ok_or_else(|| "evaluation manifest has no development report".to_owned())?;
    let development_bytes = read_ref(root, base, development_ref)?;
    require_hash(
        &development_bytes,
        DEVELOPMENT_REPORT_SHA256,
        "axis-development report",
    )?;
    validate_development_report(&development_bytes)?;
    let rule = manifest
        .evaluation_rule
        .as_ref()
        .ok_or_else(|| "evaluation manifest has no rule".to_owned())?;
    let candidate = candidate_by_id(FIXED_CANDIDATE_ID)?;

    // Both object profiles and the immutable evaluation rule are validated
    // before either previously unopened listener block is fetched.
    let blue_profile = frozen_profile(BLUE_BOWL_PROFILE_ID)?;
    let blue = acquire_selected_conditions(blue_profile, &manifest.conditions)?;
    let blue_modes = modes_for(&transfer, BLUE_BOWL_PROFILE_ID)?;
    let blue_report = evaluate_object(&blue, &blue_modes, candidate, rule)?;

    // Glass remains an equally weighted second holdout; no selection or rule
    // changes are possible after observing Blue.
    let glass_profile = frozen_profile(GLASS_GOBLET_PROFILE_ID)?;
    let glass = acquire_selected_conditions(glass_profile, &manifest.conditions)?;
    let glass_modes = modes_for(&transfer, GLASS_GOBLET_PROFILE_ID)?;
    let glass_report = evaluate_object(&glass, &glass_modes, candidate, rule)?;
    let every_object_passed = blue_report.object_gate.passed && glass_report.object_gate.passed;
    let decision = if every_object_passed {
        "FixedVerticalCandidateTwoObjectAxisStratificationSupported"
    } else {
        "FixedVerticalCandidateTwoObjectAxisStratificationRejected"
    };
    let report = EvaluationReport {
        schema: "nextengine.experimental-realimpact-spatial-axis-evaluation.report.v1",
        status: "Validated",
        decision,
        claim: "FIXED_RBF_VERTICAL_WITHIN_LINE_ROBUSTNESS_ACROSS_TWO_OBJECTS_AND_SIX_DECLARED_CONDITIONS_ONLY / NO_ANGLE_DISTANCE_INTERPOLATION_3D_FIELD_MATERIAL_QUALITY_ADMISSION_OR_RUNTIME_AUTHORITY",
        study_id: STUDY_ID,
        revision: EVALUATION_REVISION,
        manifest_sha256: EVALUATION_MANIFEST_SHA256,
        spatial_calibration_report_sha256: SPATIAL_REPORT_SHA256,
        transfer_calibration_report_sha256: TRANSFER_REPORT_SHA256,
        development_report_sha256: DEVELOPMENT_REPORT_SHA256,
        fixed_candidate_id: FIXED_CANDIDATE_ID,
        evaluation_rule: rule,
        object_reports: vec![blue_report, glass_report],
        every_object_passed,
        allowed_claims: &EVALUATION_ALLOWED_CLAIMS,
        prohibited_claims: &EVALUATION_PROHIBITED_CLAIMS,
        next_action: "if supported, preregister a true coordinate-space angle/distance interpolation model; if rejected, retain the vertical-line RBF only and add no 3D representation",
    };
    let report_bytes = pretty_json(&report)?;
    let report_sha256 = sha256_hex(&report_bytes);
    let output = resolve_output_path(root, output_argument)?;
    require_empty_output(&output)?;
    publish::evaluation(
        &output,
        &manifest_bytes,
        &spatial_bytes,
        &transfer_bytes,
        &development_bytes,
        &blue.payload,
        &glass.payload,
        &report_bytes,
    )?;
    println!("REALIMPACT spatial axis evaluation: {}", output.display());
    println!("evaluation manifest sha256: {EVALUATION_MANIFEST_SHA256}");
    println!("Blue selected block sha256: {}", blue.payload_sha256);
    println!("Glass selected block sha256: {}", glass.payload_sha256);
    println!("every object passed: {every_object_passed}");
    println!("report decision: {decision}");
    println!("report sha256: {report_sha256}");
    Ok(())
}

fn evaluate_object<'a>(
    acquisition: &'a Acquisition,
    modes: &[ModeSeed],
    candidate: CandidateProfile,
    rule: &EvaluationRule,
) -> Result<ObjectEvaluation<'a>, String> {
    let condition_reports = acquisition
        .conditions
        .iter()
        .map(|block| {
            let evaluation = dsp::evaluate(&block.rows, modes, SAMPLE_RATE_HZ, candidate)?;
            Ok(ConditionReport {
                condition: block.condition.clone(),
                gate: gate(&evaluation),
                evaluation,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let aggregate = aggregate(&condition_reports)?;
    let object_gate = object_gate(&aggregate, rule);
    Ok(ObjectEvaluation {
        object_profile: acquisition.profile.id,
        condition_reports,
        aggregate,
        object_gate,
        acquisition: acquisition.summary(),
    })
}

fn object_gate(aggregate: &Aggregate, rule: &EvaluationRule) -> Gate {
    let checks = vec![
        GateCheck {
            metric: "base_condition_passed",
            observed: if aggregate.base_condition_passed {
                1.0
            } else {
                0.0
            },
            relation: "==",
            threshold: if rule.require_base_condition_pass {
                1.0
            } else {
                0.0
            },
            passed: aggregate.base_condition_passed == rule.require_base_condition_pass,
        },
        check(
            "passed_condition_count",
            aggregate.passed_condition_count as f64,
            ">=",
            rule.minimum_passed_condition_count as f64,
        ),
        check(
            "median_condition_median_error_db",
            aggregate.median_condition_median_error_db,
            "<=",
            rule.maximum_median_condition_median_error_db,
        ),
        check(
            "median_condition_ratio_to_constant",
            aggregate.median_condition_ratio_to_constant,
            "<=",
            rule.maximum_median_condition_ratio_to_constant,
        ),
        check(
            "worst_condition_p90_error_db",
            aggregate.worst_condition_p90_error_db,
            "<=",
            rule.maximum_worst_condition_p90_error_db,
        ),
    ];
    Gate {
        passed: checks.iter().all(|check| check.passed),
        checks,
    }
}

fn validate_development_manifest(manifest: &Manifest) -> Result<(), String> {
    if manifest.schema != MANIFEST_SCHEMA
        || manifest.study_id != STUDY_ID
        || manifest.revision != DEVELOPMENT_REVISION
        || manifest.phase != "development"
        || manifest.spatial_calibration_report.sha256 != SPATIAL_REPORT_SHA256
        || manifest.transfer_calibration_report.sha256 != TRANSFER_REPORT_SHA256
        || manifest.development_report.is_some()
        || manifest.fixed_candidate_id != FIXED_CANDIDATE_ID
        || manifest.compressed_prefix_bytes != PREFIX_BYTES
        || manifest.object_profiles != [GREEN_GOBLET_PROFILE_ID]
        || manifest.conditions != frozen_conditions()
        || manifest.condition_gate != frozen_gates()
        || manifest.evaluation_rule.is_some()
        || manifest.prior_exposures.is_some()
        || manifest.allowed_claims != ALLOWED_CLAIMS
        || manifest.prohibited_claims != PROHIBITED_CLAIMS
    {
        return Err(
            "spatial-extension development manifest does not match the frozen protocol".to_owned(),
        );
    }
    Ok(())
}

fn validate_evaluation_manifest(manifest: &Manifest) -> Result<(), String> {
    if manifest.schema != MANIFEST_SCHEMA
        || manifest.study_id != STUDY_ID
        || manifest.revision != EVALUATION_REVISION
        || manifest.phase != "evaluation"
        || manifest.spatial_calibration_report.sha256 != SPATIAL_REPORT_SHA256
        || manifest.transfer_calibration_report.sha256 != TRANSFER_REPORT_SHA256
        || manifest
            .development_report
            .as_ref()
            .map(|reference| reference.sha256.as_str())
            != Some(DEVELOPMENT_REPORT_SHA256)
        || manifest.fixed_candidate_id != FIXED_CANDIDATE_ID
        || manifest.compressed_prefix_bytes != PREFIX_BYTES
        || manifest.object_profiles != [BLUE_BOWL_PROFILE_ID, GLASS_GOBLET_PROFILE_ID]
        || manifest.conditions != frozen_conditions()
        || manifest.condition_gate != frozen_gates()
        || manifest.evaluation_rule.as_ref() != Some(&frozen_evaluation_rule())
        || !string_values_equal(
            manifest.prior_exposures.as_deref(),
            &EVALUATION_PRIOR_EXPOSURES,
        )
        || manifest.allowed_claims != EVALUATION_ALLOWED_CLAIMS
        || manifest.prohibited_claims != EVALUATION_PROHIBITED_CLAIMS
    {
        return Err(
            "spatial-extension evaluation manifest does not match the frozen protocol".to_owned(),
        );
    }
    Ok(())
}

fn validate_development_report(bytes: &[u8]) -> Result<(), String> {
    let report: DevelopmentPrerequisite = serde_json::from_slice(bytes)
        .map_err(|error| format!("parse axis-development report: {error}"))?;
    if report.schema != REPORT_SCHEMA
        || report.status != "Validated"
        || report.decision != "DevelopmentAxisStratificationMeasured"
        || report.manifest_sha256 != DEVELOPMENT_MANIFEST_SHA256
        || report.fixed_candidate_id != FIXED_CANDIDATE_ID
        || report.aggregate.condition_count != 6
        || report.aggregate.passed_condition_count != 6
        || !report.aggregate.base_condition_passed
    {
        return Err("axis-development report does not authorize holdout evaluation".to_owned());
    }
    Ok(())
}

fn frozen_evaluation_rule() -> EvaluationRule {
    EvaluationRule {
        require_base_condition_pass: true,
        minimum_passed_condition_count: 4,
        maximum_median_condition_median_error_db: 7.0,
        maximum_median_condition_ratio_to_constant: 0.9,
        maximum_worst_condition_p90_error_db: 24.0,
        require_every_object_pass: true,
    }
}

fn string_values_equal(values: Option<&[String]>, expected: &[&str]) -> bool {
    values.is_some_and(|values| {
        values.len() == expected.len()
            && values
                .iter()
                .zip(expected)
                .all(|(value, expected)| value == expected)
    })
}

fn validate_spatial_report(bytes: &[u8]) -> Result<(), String> {
    let report: SpatialReport = serde_json::from_slice(bytes)
        .map_err(|error| format!("parse spatial-calibration report: {error}"))?;
    if report.schema != "nextengine.experimental-realimpact-spatial-calibration.report.v1"
        || report.status != "Validated"
        || report.decision != "RelativeVerticalSpectralParticipationSupported"
        || report.selected_candidate_id != FIXED_CANDIDATE_ID
        || !report.skull_holdout.gate.passed
    {
        return Err(
            "spatial-calibration report does not freeze the extension candidate".to_owned(),
        );
    }
    Ok(())
}

fn frozen_conditions() -> Vec<ConditionSpec> {
    [
        ("angle000-distance000", 0, 0, 0),
        ("angle000-distance333", 0, 333, 15),
        ("angle000-distance666", 0, 666, 30),
        ("angle000-distance1000", 0, 1_000, 45),
        ("angle020-distance000", 20, 0, 60),
        ("angle040-distance000", 40, 0, 120),
    ]
    .into_iter()
    .map(
        |(id, azimuth_degrees, distance_offset_millimetres, row_start)| ConditionSpec {
            id: id.to_owned(),
            azimuth_degrees,
            distance_offset_millimetres,
            row_start,
        },
    )
    .collect()
}

fn acquire_selected_conditions(
    profile: &'static FrozenProfile,
    conditions: &[ConditionSpec],
) -> Result<Acquisition, String> {
    let resolve = resolve_public_https_endpoint(profile.archive_url)?
        .ok_or_else(|| "REALIMPACT archive host did not resolve".to_owned())?;
    let mut fetched_bytes = 0_u64;
    let eocd = fetch_range(
        profile,
        &resolve,
        profile.archive_bytes - 22,
        22,
        &mut fetched_bytes,
    )?;
    validate_archive_headers(profile, &eocd)?;
    validate_eocd(profile, &eocd.body)?;
    let central = fetch_range(
        profile,
        &resolve,
        profile.central_offset,
        profile.central_bytes,
        &mut fetched_bytes,
    )?;
    validate_archive_headers(profile, &central)?;
    require_hash(
        &central.body,
        profile.central_sha256,
        "ZIP central directory",
    )?;
    validate_central_entries(profile, &parse_central_directory(profile, &central.body)?)?;

    let mut raw_entries = BTreeMap::new();
    for spec in profile
        .entries
        .iter()
        .filter(|entry| !entry.raw_sha256.is_empty())
    {
        let response = fetch_entry(
            profile,
            &resolve,
            spec,
            spec.compressed_bytes,
            &mut fetched_bytes,
        )?;
        raw_entries.insert(spec.name, decompress_complete(spec, &response)?);
    }
    validate_condition_identities(profile, &raw_entries, conditions)?;
    let audio_spec = entry(profile, profile.audio_entry_name)?;
    let compressed = fetch_entry(
        profile,
        &resolve,
        audio_spec,
        PREFIX_BYTES,
        &mut fetched_bytes,
    )?;
    let compressed_prefix_sha256 = sha256_hex(&compressed);
    let (blocks, payload, row_sha256) = extract_condition_blocks(profile, &compressed, conditions)?;
    let payload_sha256 = sha256_hex(&payload);
    Ok(Acquisition {
        profile,
        payload,
        payload_sha256,
        compressed_prefix_sha256,
        row_sha256,
        fetched_bytes,
        conditions: blocks,
    })
}

type ExtractedConditions = (Vec<ConditionBlock>, Vec<u8>, Vec<RowIdentity>);

fn extract_condition_blocks(
    profile: &FrozenProfile,
    compressed: &[u8],
    conditions: &[ConditionSpec],
) -> Result<ExtractedConditions, String> {
    let last_row = conditions
        .iter()
        .map(|condition| condition.row_start + LISTENER_COUNT)
        .max()
        .ok_or_else(|| "spatial-extension has no conditions".to_owned())?;
    let mut decoder = DeflateDecoder::new(compressed);
    let mut header = [0_u8; 128];
    decoder
        .read_exact(&mut header)
        .map_err(|error| format!("decompress REALIMPACT transfer NPY header: {error}"))?;
    validate_npy_header_prefix(&header, "<f4", &[3_000, profile.audio_sample_count])?;
    let mut rows = BTreeMap::new();
    let mut payload = Vec::new();
    let mut row_sha256 = Vec::new();
    for row_index in 0..last_row {
        let mut bytes = vec![0_u8; profile.audio_sample_count * 4];
        decoder
            .read_exact(&mut bytes)
            .map_err(|error| format!("decompress REALIMPACT transfer row {row_index}: {error}"))?;
        let selected = conditions.iter().any(|condition| {
            (condition.row_start..condition.row_start + LISTENER_COUNT).contains(&row_index)
        });
        if selected {
            let payload_offset_bytes = payload.len();
            let sha256 = sha256_hex(&bytes);
            rows.insert(row_index, decode_row(&bytes, row_index)?);
            payload.extend_from_slice(&bytes);
            row_sha256.push(RowIdentity {
                row_index,
                payload_offset_bytes,
                sha256,
            });
        }
    }
    let blocks = conditions
        .iter()
        .map(|condition| {
            let rows = (condition.row_start..condition.row_start + LISTENER_COUNT)
                .map(|row_index| {
                    rows.remove(&row_index)
                        .ok_or_else(|| format!("selected row {row_index} was not decoded"))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(ConditionBlock {
                condition: condition.clone(),
                rows,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok((blocks, payload, row_sha256))
}

fn validate_condition_identities(
    profile: &FrozenProfile,
    raw_entries: &BTreeMap<&'static str, Vec<u8>>,
    conditions: &[ConditionSpec],
) -> Result<(), String> {
    let vertex_xyz = f64_array(raw(raw_entries, "vertexXYZ.npy")?, &[3_000, 3])?;
    let listener_xyz = f64_array(raw(raw_entries, "listenerXYZ.npy")?, &[3_000, 3])?;
    let vertex_ids = i64_array(raw(raw_entries, "vertexID.npy")?, &[3_000])?;
    let microphones = i64_array(raw(raw_entries, "micID.npy")?, &[3_000])?;
    let distances = i64_array(raw(raw_entries, "distance.npy")?, &[3_000])?;
    let angles = i64_array(raw(raw_entries, "angle.npy")?, &[3_000])?;
    for condition in conditions {
        for microphone in 0..LISTENER_COUNT {
            let row = condition.row_start + microphone;
            let offset = row * 3;
            let expected_listener = listener_position(condition, microphone);
            if vertex_ids[row] != profile.expected_impact_vertex_id as i64
                || !approximately_equal(
                    &vertex_xyz[offset..offset + 3],
                    &profile.expected_impact_position,
                )
                || microphones[row] != microphone as i64
                || distances[row] != i64::from(condition.distance_offset_millimetres)
                || angles[row] != i64::from(condition.azimuth_degrees)
                || !approximately_equal(&listener_xyz[offset..offset + 3], &expected_listener)
            {
                return Err(format!(
                    "REALIMPACT {} condition identity changed at row {row}",
                    profile.dataset_object_id
                ));
            }
        }
    }
    Ok(())
}

fn listener_position(condition: &ConditionSpec, microphone: usize) -> [f64; 3] {
    let x = 0.23 + f64::from(condition.distance_offset_millimetres) / 1_000.0;
    let y = -0.04345_f64;
    let z = -0.91 + microphone as f64 / 14.0 * 1.82;
    let angle = f64::from(condition.azimuth_degrees) * PI / 180.0;
    [
        angle.cos() * x - angle.sin() * y,
        angle.sin() * x + angle.cos() * y,
        z,
    ]
}

fn approximately_equal(left: &[f64], right: &[f64]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| (left - right).abs() <= 1.0e-12)
}

fn aggregate(reports: &[ConditionReport]) -> Result<Aggregate, String> {
    let condition_count = reports.len();
    if condition_count == 0 {
        return Err("spatial-extension has no condition reports".to_owned());
    }
    let passed_condition_count = reports.iter().filter(|report| report.gate.passed).count();
    let medians = reports
        .iter()
        .map(|report| report.evaluation.median_abs_error_db)
        .collect::<Vec<_>>();
    let ratios = reports
        .iter()
        .map(|report| report.evaluation.median_error_ratio_to_constant)
        .collect::<Vec<_>>();
    Ok(Aggregate {
        condition_count,
        passed_condition_count,
        passed_condition_fraction: passed_condition_count as f64 / condition_count as f64,
        base_condition_passed: reports[0].gate.passed,
        median_condition_median_error_db: median(&medians)?,
        median_condition_ratio_to_constant: median(&ratios)?,
        worst_condition_p90_error_db: reports
            .iter()
            .map(|report| report.evaluation.p90_abs_error_db)
            .max_by(f64::total_cmp)
            .expect("non-empty reports"),
    })
}

fn median(values: &[f64]) -> Result<f64, String> {
    let mut values = values.to_vec();
    values.sort_by(f64::total_cmp);
    match values.len() {
        0 => Err("spatial-extension aggregate is empty".to_owned()),
        length if length % 2 == 1 => Ok(values[length / 2]),
        length => Ok((values[length / 2 - 1] + values[length / 2]) * 0.5),
    }
}

struct Acquisition {
    profile: &'static FrozenProfile,
    payload: Vec<u8>,
    payload_sha256: String,
    compressed_prefix_sha256: String,
    row_sha256: Vec<RowIdentity>,
    fetched_bytes: u64,
    conditions: Vec<ConditionBlock>,
}

impl Acquisition {
    fn summary(&self) -> AcquisitionSummary<'_> {
        let selected_block_path = match self.profile.id {
            GREEN_GOBLET_PROFILE_ID => SELECTED_BLOCK_FILE,
            BLUE_BOWL_PROFILE_ID => "blue-axis-selected-blocks.f32le",
            GLASS_GOBLET_PROFILE_ID => "glass-axis-selected-blocks.f32le",
            _ => "unsupported-axis-selected-blocks.f32le",
        };
        AcquisitionSummary {
            dataset_object_id: self.profile.dataset_object_id,
            archive_url: self.profile.archive_url,
            archive_content_length: self.profile.archive_bytes,
            compressed_prefix_bytes: PREFIX_BYTES,
            compressed_prefix_sha256: &self.compressed_prefix_sha256,
            http_range_payload_bytes: self.fetched_bytes,
            full_archive_fraction: self.fetched_bytes as f64 / self.profile.archive_bytes as f64,
            selected_block_path,
            selected_block_sha256: &self.payload_sha256,
            selected_row_count: self.row_sha256.len(),
            selected_rows: &self.row_sha256,
        }
    }
}

struct ConditionBlock {
    condition: ConditionSpec,
    rows: Vec<Vec<f64>>,
}

#[derive(Deserialize)]
struct PhaseHeader {
    phase: String,
}

#[derive(Deserialize)]
struct Manifest {
    schema: String,
    study_id: String,
    revision: String,
    phase: String,
    spatial_calibration_report: FileRef,
    transfer_calibration_report: FileRef,
    development_report: Option<FileRef>,
    fixed_candidate_id: String,
    compressed_prefix_bytes: u64,
    object_profiles: Vec<String>,
    conditions: Vec<ConditionSpec>,
    condition_gate: Gates,
    evaluation_rule: Option<EvaluationRule>,
    #[serde(default)]
    prior_exposures: Option<Vec<String>>,
    allowed_claims: Vec<String>,
    prohibited_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
struct ConditionSpec {
    id: String,
    azimuth_degrees: u32,
    distance_offset_millimetres: u32,
    row_start: usize,
}

#[derive(Deserialize)]
struct SpatialReport {
    schema: String,
    status: String,
    decision: String,
    selected_candidate_id: String,
    skull_holdout: SpatialHoldout,
}

#[derive(Deserialize)]
struct SpatialHoldout {
    gate: SpatialGate,
}

#[derive(Deserialize)]
struct SpatialGate {
    passed: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
struct EvaluationRule {
    require_base_condition_pass: bool,
    minimum_passed_condition_count: usize,
    maximum_median_condition_median_error_db: f64,
    maximum_median_condition_ratio_to_constant: f64,
    maximum_worst_condition_p90_error_db: f64,
    require_every_object_pass: bool,
}

#[derive(Deserialize)]
struct DevelopmentPrerequisite {
    schema: String,
    status: String,
    decision: String,
    manifest_sha256: String,
    fixed_candidate_id: String,
    aggregate: DevelopmentAggregate,
}

#[derive(Deserialize)]
struct DevelopmentAggregate {
    condition_count: usize,
    passed_condition_count: usize,
    base_condition_passed: bool,
}

#[derive(Serialize)]
struct ConditionReport {
    condition: ConditionSpec,
    gate: Gate,
    evaluation: Evaluation,
}

#[derive(Serialize)]
struct Aggregate {
    condition_count: usize,
    passed_condition_count: usize,
    passed_condition_fraction: f64,
    base_condition_passed: bool,
    median_condition_median_error_db: f64,
    median_condition_ratio_to_constant: f64,
    worst_condition_p90_error_db: f64,
}

#[derive(Serialize)]
struct RowIdentity {
    row_index: usize,
    payload_offset_bytes: usize,
    sha256: String,
}

#[derive(Serialize)]
struct AcquisitionSummary<'a> {
    dataset_object_id: &'static str,
    archive_url: &'static str,
    archive_content_length: u64,
    compressed_prefix_bytes: u64,
    compressed_prefix_sha256: &'a str,
    http_range_payload_bytes: u64,
    full_archive_fraction: f64,
    selected_block_path: &'static str,
    selected_block_sha256: &'a str,
    selected_row_count: usize,
    selected_rows: &'a [RowIdentity],
}

#[derive(Serialize)]
struct DevelopmentReport<'a> {
    schema: &'static str,
    status: &'static str,
    decision: &'static str,
    claim: &'static str,
    study_id: &'static str,
    revision: &'static str,
    manifest_sha256: &'static str,
    spatial_calibration_report_sha256: &'static str,
    transfer_calibration_report_sha256: &'static str,
    fixed_candidate_id: &'static str,
    object_profile: &'static str,
    condition_reports: Vec<ConditionReport>,
    aggregate: Aggregate,
    acquisition: AcquisitionSummary<'a>,
    allowed_claims: &'static [&'static str],
    prohibited_claims: &'static [&'static str],
    next_action: &'static str,
}

#[derive(Serialize)]
struct ObjectEvaluation<'a> {
    object_profile: &'static str,
    condition_reports: Vec<ConditionReport>,
    aggregate: Aggregate,
    object_gate: Gate,
    acquisition: AcquisitionSummary<'a>,
}

#[derive(Serialize)]
struct EvaluationReport<'a> {
    schema: &'static str,
    status: &'static str,
    decision: &'static str,
    claim: &'static str,
    study_id: &'static str,
    revision: &'static str,
    manifest_sha256: &'static str,
    spatial_calibration_report_sha256: &'static str,
    transfer_calibration_report_sha256: &'static str,
    development_report_sha256: &'static str,
    fixed_candidate_id: &'static str,
    evaluation_rule: &'a EvaluationRule,
    object_reports: Vec<ObjectEvaluation<'a>>,
    every_object_passed: bool,
    allowed_claims: &'static [&'static str],
    prohibited_claims: &'static [&'static str],
    next_action: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_conditions_cover_distance_and_angle_strata() {
        let conditions = frozen_conditions();
        assert_eq!(conditions.len(), 6);
        assert_eq!(conditions[3].distance_offset_millimetres, 1_000);
        assert_eq!(conditions[5].azimuth_degrees, 40);
        assert_eq!(conditions[5].row_start, 120);
    }

    #[test]
    fn official_listener_transform_preserves_height() {
        let condition = frozen_conditions().pop().expect("condition");
        let bottom = listener_position(&condition, 0);
        let top = listener_position(&condition, 14);
        assert!((bottom[2] + 0.91).abs() < 1.0e-12);
        assert!((top[2] - 0.91).abs() < 1.0e-12);
        assert!((bottom[0].hypot(bottom[1]) - 0.234_068_157_808_788_7).abs() < 1.0e-12);
    }
}
