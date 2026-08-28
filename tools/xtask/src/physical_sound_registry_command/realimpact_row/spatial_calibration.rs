use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use flate2::read::DeflateDecoder;
use serde::{Deserialize, Serialize};

use self::dsp::{
    ANCHOR_LISTENERS, CANDIDATES, CandidateProfile, Evaluation, HELD_LISTENERS, LISTENER_COUNT,
    ModeSeed, ONSET_PEAK_FRACTION, REFERENCE_LISTENER, WINDOW_SAMPLES,
};
use super::*;

pub(in super::super) mod dsp;
mod extension;
mod shape;
pub(in crate::physical_sound_registry_command) use shape::run_cli as run_shape;

pub(super) fn run_extension_cli(
    root: &Path,
    arguments: impl Iterator<Item = String>,
) -> Result<(), String> {
    extension::run_cli(root, arguments)
}

const MANIFEST_SCHEMA: &str = "nextengine.experimental-realimpact-spatial-calibration.manifest.v1";
const REPORT_SCHEMA: &str = "nextengine.experimental-realimpact-spatial-calibration.report.v1";
const STUDY_ID: &str = "physical-sound-realimpact-vertical-spectral-interpolation";
const STUDY_REVISION: &str = "v1";
const PREREGISTRATION_MANIFEST_SHA256: &str =
    "077b9a468aefb4b55a6f06585dc5085dc2b0b04a3fe9468a20ed5d1383d3f2cb";
const TRANSFER_REPORT_SHA256: &str =
    "01346767b596630061fe437e98d5213bf426e49e5b96c22565a77acfea50d444";
const GREEN_MANIFEST_SHA256: &str =
    "fcf44d41bdd54ad3bc9df27b6c8ccc4d5ccd470a1f786641e64461e794c850de";
const GREEN_BLOCK_SHA256: &str = "8bcffd0a9f57fd101a803228f7e8aa66d469f82d99ad6e43950318aae0f875ca";
const COMPRESSED_PREFIX_BYTES: u64 = 16 * 1024 * 1024;
const SAMPLE_RATE_HZ: u32 = 48_000;
const MAX_MANIFEST_BYTES: usize = 1024 * 1024;
const LISTENER_Z_METRES: [f64; LISTENER_COUNT] = [
    -0.91, -0.78, -0.65, -0.52, -0.39, -0.26, -0.13, 0.0, 0.13, 0.26, 0.39, 0.52, 0.65, 0.78, 0.91,
];
const ALLOWED_CLAIMS: [&str; 2] = [
    "fixed_angle_distance_relative_vertical_spectral_interpolation",
    "object_disjoint_holdout_result",
];
const PROHIBITED_CLAIMS: [&str; 10] = [
    "absolute_amplitude_claim",
    "arbitrary_angle_interpolation",
    "arbitrary_distance_interpolation",
    "exact_material_composition_claim",
    "naturalness_or_perceptual_quality",
    "physical_modal_radiation_identity",
    "physical_sound_pass",
    "production_corpus_admission",
    "runtime_content_role",
    "three_dimensional_spatial_field",
];
const PRIOR_EXPOSURES: [&str; 3] = [
    "Green Goblet listener rows 0..14 were used for candidate and gate development",
    "Shell Plate row 0 was previously used by the modal/damping calibration",
    "Skull Cup row 0 was previously used only by the modal/damping holdout; listener rows 1..14 remained unopened before this preregistration",
];

const MIN_COMPONENT_COUNT: usize = 12;
const MAX_MEDIAN_ERROR_DB: f64 = 7.0;
const MAX_P90_ERROR_DB: f64 = 18.0;
const MAX_PERSISTENT_MEDIAN_ERROR_DB: f64 = 8.0;
const MIN_IMPROVED_COMPONENT_FRACTION: f64 = 0.5;
const MAX_MEDIAN_RATIO_TO_CONSTANT: f64 = 0.9;

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
            _ => return Err(format!("unexpected spatial-calibration argument: {flag}")),
        }
    }
    let manifest = manifest.ok_or_else(|| {
        "physical-sound-registry spatial-calibration requires --manifest <external-json>".to_owned()
    })?;
    let output = output.ok_or_else(|| {
        "physical-sound-registry spatial-calibration requires --output <external-empty-directory>"
            .to_owned()
    })?;
    run(root, &manifest, &output)
}

fn run(root: &Path, manifest_argument: &Path, output_argument: &Path) -> Result<(), String> {
    let manifest_path = canonical_external_file(
        root,
        &resolve_cli_path(root, manifest_argument),
        "REALIMPACT spatial preregistration manifest",
    )?;
    let manifest_bytes = read_bounded_file(
        &manifest_path,
        MAX_MANIFEST_BYTES,
        "REALIMPACT spatial preregistration manifest",
    )?;
    require_hash(
        &manifest_bytes,
        PREREGISTRATION_MANIFEST_SHA256,
        "spatial preregistration manifest",
    )?;
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("parse spatial preregistration manifest: {error}"))?;
    validate_manifest(&manifest)?;
    let manifest_sha256 = sha256_hex(&manifest_bytes);
    let base = manifest_path
        .parent()
        .ok_or_else(|| "spatial manifest has no parent".to_owned())?;
    let transfer_bytes = read_ref(root, base, &manifest.transfer_calibration_report)?;
    require_hash(
        &transfer_bytes,
        TRANSFER_REPORT_SHA256,
        "transfer-calibration report",
    )?;
    let transfer: TransferReport = serde_json::from_slice(&transfer_bytes)
        .map_err(|error| format!("parse transfer-calibration report: {error}"))?;
    validate_transfer(&transfer)?;

    let green_manifest_bytes = read_ref(root, base, &manifest.green_development_manifest)?;
    require_hash(
        &green_manifest_bytes,
        GREEN_MANIFEST_SHA256,
        "Green development manifest",
    )?;
    let green_manifest: GreenManifest = serde_json::from_slice(&green_manifest_bytes)
        .map_err(|error| format!("parse Green listener manifest: {error}"))?;
    validate_green_manifest(&green_manifest)?;
    let green_block = read_ref(root, base, &manifest.green_development_block)?;
    require_hash(&green_block, GREEN_BLOCK_SHA256, "Green listener block")?;
    let green_rows = rows_from_block(&green_block, green_manifest.sample_count_per_row)?;
    let green_modes = modes_for(&transfer, GREEN_GOBLET_PROFILE_ID)?;
    let development = evaluate_candidates(&green_rows, &green_modes)?;
    if development.iter().any(|result| !result.gate.passed) {
        return Err("a preregistered candidate failed the Green development gate; Shell and Skull remain unopened".to_owned());
    }

    let output = resolve_output_path(root, output_argument)?;
    require_empty_output(&output)?;

    // The calibration object is intentionally opened only after every manifest,
    // development and gate check above has passed.
    let shell = acquire_block(frozen_profile(SHELL_PLATE_PROFILE_ID)?)?;
    let shell_modes = modes_for(&transfer, SHELL_PLATE_PROFILE_ID)?;
    let calibration = evaluate_candidates(&shell.rows, &shell_modes)?;
    let selected = select_candidate(&development, &calibration)?;
    let selected_candidate_id = selected.candidate_id;
    let selected_calibration_loss = selected.evaluation.calibration_loss;
    let selected_calibration_gate_passed = selected.gate.passed;
    let selection = SelectionSnapshot {
        schema: "nextengine.experimental-realimpact-spatial-selection.v1",
        preregistration_manifest_sha256: &manifest_sha256,
        selected_candidate_id,
        selected_calibration_loss,
        shell_calibration_gate_passed: selected_calibration_gate_passed,
        skull_holdout_opened: false,
    };
    let selection_bytes = pretty_json(&selection)?;
    let selection_sha256 = sha256_hex(&selection_bytes);

    // The holdout is opened only after the selection snapshot exists in memory
    // and has a stable hash. No candidate can be changed after this point.
    let skull = acquire_block(frozen_profile(SKULL_CUP_PROFILE_ID)?)?;
    let skull_modes = modes_for(&transfer, SKULL_CUP_PROFILE_ID)?;
    let candidate = candidate_by_id(selected_candidate_id)?;
    let holdout_evaluation = dsp::evaluate(&skull.rows, &skull_modes, SAMPLE_RATE_HZ, candidate)?;
    let holdout_gate = gate(&holdout_evaluation);
    let decision = if holdout_gate.passed {
        "RelativeVerticalSpectralParticipationSupported"
    } else {
        "RelativeVerticalSpectralParticipationRejected"
    };
    let report = Report {
        schema: REPORT_SCHEMA,
        status: "Validated",
        decision,
        claim: "FIXED_ANGLE_DISTANCE_RELATIVE_VERTICAL_SELECTED_SPECTRAL_PARTICIPATION_ONLY / NO_3D_FIELD_ABSOLUTE_AMPLITUDE_NATURALNESS_ADMISSION_OR_RUNTIME_AUTHORITY",
        study_id: STUDY_ID,
        revision: STUDY_REVISION,
        preregistration_manifest_sha256: &manifest_sha256,
        transfer_calibration_report_sha256: TRANSFER_REPORT_SHA256,
        green_development_manifest_sha256: GREEN_MANIFEST_SHA256,
        green_development_block_sha256: GREEN_BLOCK_SHA256,
        development,
        shell_calibration: calibration,
        selection_sha256: &selection_sha256,
        selected_candidate_id,
        skull_holdout: CandidateEvaluation {
            candidate_id: selected_candidate_id,
            gate: holdout_gate,
            evaluation: holdout_evaluation,
        },
        shell_acquisition: shell.summary(),
        skull_acquisition: skull.summary(),
        holdout_opened_after_selection_hash: true,
        allowed_claims: &ALLOWED_CLAIMS,
        prohibited_claims: &PROHIBITED_CLAIMS,
        remaining_spatial_scope: "three-dimensional, arbitrary-angle and arbitrary-distance radiation remain unmeasured",
        next_action: "test the same frozen candidate on at least two additional object-disjoint impact blocks and add angle/distance axes before proposing a runtime representation",
    };
    let report_bytes = pretty_json(&report)?;
    publish(
        &output,
        &manifest_bytes,
        &transfer_bytes,
        &green_manifest_bytes,
        &selection_bytes,
        &shell,
        &skull,
        &report_bytes,
    )?;
    println!(
        "REALIMPACT spatial calibration output: {}",
        output.display()
    );
    println!("preregistration manifest sha256: {manifest_sha256}");
    println!("selection sha256: {selection_sha256}");
    println!("selected candidate: {selected_candidate_id}");
    println!("holdout gate passed: {}", report.skull_holdout.gate.passed);
    println!("report decision: {decision}");
    Ok(())
}

fn read_ref(root: &Path, base: &Path, reference: &FileRef) -> Result<Vec<u8>, String> {
    let path = canonical_external_file(
        root,
        &if reference.path.is_absolute() {
            reference.path.clone()
        } else {
            base.join(&reference.path)
        },
        "spatial prerequisite",
    )?;
    let bytes = read_bounded_file(&path, 128 * 1024 * 1024, "spatial prerequisite")?;
    require_hash(&bytes, &reference.sha256, "spatial prerequisite")?;
    Ok(bytes)
}

fn validate_manifest(manifest: &Manifest) -> Result<(), String> {
    let candidates = CANDIDATES
        .iter()
        .map(|candidate| candidate.id)
        .collect::<Vec<_>>();
    if manifest.schema != MANIFEST_SCHEMA
        || manifest.study_id != STUDY_ID
        || manifest.revision != STUDY_REVISION
        || manifest.object_split.development != "93_GreenGoblet"
        || manifest.object_split.calibration != "51_ShellPlate"
        || manifest.object_split.holdout != "60_SkullCup"
        || manifest.anchor_listeners != ANCHOR_LISTENERS
        || manifest.held_listeners != HELD_LISTENERS
        || manifest.reference_listener != REFERENCE_LISTENER
        || manifest.window_samples != WINDOW_SAMPLES
        || manifest.onset_peak_fraction != ONSET_PEAK_FRACTION
        || manifest.candidate_ids != candidates
        || manifest.gates != frozen_gates()
        || manifest.allowed_claims != ALLOWED_CLAIMS
        || manifest.prohibited_claims != PROHIBITED_CLAIMS
        || manifest.prior_exposures != PRIOR_EXPOSURES
        || manifest.shell_archive != archive_range(frozen_profile(SHELL_PLATE_PROFILE_ID)?)
        || manifest.skull_archive != archive_range(frozen_profile(SKULL_CUP_PROFILE_ID)?)
        || manifest.transfer_calibration_report.sha256 != TRANSFER_REPORT_SHA256
        || manifest.green_development_manifest.sha256 != GREEN_MANIFEST_SHA256
        || manifest.green_development_block.sha256 != GREEN_BLOCK_SHA256
    {
        return Err(
            "spatial preregistration manifest does not match the frozen v1 protocol".to_owned(),
        );
    }
    Ok(())
}

fn validate_transfer(report: &TransferReport) -> Result<(), String> {
    if report.schema != "nextengine.experimental-realimpact-transfer-calibration.report.v2"
        || report.status != "Validated"
        || report.decision != "RelativeModalDampingSupportedSpatialUnavailable"
        || report.selected_profile_id != "injective-modal-16-fft65536-v2"
        || report
            .selected_rows
            .iter()
            .filter(|row| row.profile_row_id == SHELL_PLATE_PROFILE_ID)
            .count()
            != 1
        || report.holdout_evaluation.profile_row_id != SKULL_CUP_PROFILE_ID
    {
        return Err(
            "transfer report does not match the preregistered spatial prerequisite".to_owned(),
        );
    }
    Ok(())
}

fn validate_green_manifest(manifest: &GreenManifest) -> Result<(), String> {
    if manifest.schema != "nextengine.experimental-realimpact-listener-block.manifest.v1"
        || manifest.status != "development_pilot_partial_source"
        || manifest.dataset_object_id != "93_GreenGoblet"
        || manifest.sample_rate_hz != SAMPLE_RATE_HZ
        || manifest.row_count != LISTENER_COUNT
        || manifest.rows.len() != LISTENER_COUNT
        || manifest.block_payload.sha256 != GREEN_BLOCK_SHA256
        || manifest.rows.iter().enumerate().any(|(index, row)| {
            row.row_index != index
                || row.microphone_id != index
                || row.listener_position_metres != [0.23, -0.04345, LISTENER_Z_METRES[index]]
        })
    {
        return Err("Green development listener manifest changed".to_owned());
    }
    Ok(())
}

fn modes_for(report: &TransferReport, profile: &str) -> Result<Vec<ModeSeed>, String> {
    let analysis = if profile == SKULL_CUP_PROFILE_ID {
        &report.holdout_evaluation.analysis
    } else {
        &report
            .selected_rows
            .iter()
            .find(|row| row.profile_row_id == profile)
            .ok_or_else(|| format!("transfer report has no {profile} analysis"))?
            .analysis
    };
    if analysis.modes.len() != 16 {
        return Err(format!("transfer report {profile} mode count changed"));
    }
    analysis
        .modes
        .iter()
        .map(|mode| {
            if !mode.frequency_hz.is_finite() || mode.frequency_hz <= 0.0 {
                return Err(format!(
                    "transfer report {profile} has invalid mode frequency"
                ));
            }
            Ok(ModeSeed {
                frequency_hz: mode.frequency_hz,
                persistent: mode.matched_tail_frequency_hz.is_some(),
            })
        })
        .collect()
}

fn rows_from_block(bytes: &[u8], sample_count: usize) -> Result<Vec<Vec<f64>>, String> {
    let row_bytes = sample_count
        .checked_mul(4)
        .ok_or_else(|| "listener row byte count overflow".to_owned())?;
    if bytes.len() != row_bytes * LISTENER_COUNT {
        return Err("listener block dimensions changed".to_owned());
    }
    bytes
        .chunks_exact(row_bytes)
        .enumerate()
        .map(|(row_index, row)| decode_row(row, row_index))
        .collect()
}

fn decode_row(bytes: &[u8], row_index: usize) -> Result<Vec<f64>, String> {
    bytes
        .chunks_exact(4)
        .map(|sample| {
            let value = f64::from(f32::from_le_bytes(sample.try_into().expect("four bytes")));
            value.is_finite().then_some(value).ok_or_else(|| {
                format!("REALIMPACT listener row {row_index} contains a non-finite sample")
            })
        })
        .collect()
}

fn evaluate_candidates(
    rows: &[Vec<f64>],
    modes: &[ModeSeed],
) -> Result<Vec<CandidateEvaluation>, String> {
    CANDIDATES
        .iter()
        .copied()
        .map(|candidate| {
            let evaluation = dsp::evaluate(rows, modes, SAMPLE_RATE_HZ, candidate)?;
            Ok(CandidateEvaluation {
                candidate_id: candidate.id,
                gate: gate(&evaluation),
                evaluation,
            })
        })
        .collect()
}

fn gate(evaluation: &Evaluation) -> Gate {
    let checks = vec![
        check(
            "component_count",
            evaluation.component_count as f64,
            ">=",
            MIN_COMPONENT_COUNT as f64,
        ),
        check(
            "median_abs_error_db",
            evaluation.median_abs_error_db,
            "<=",
            MAX_MEDIAN_ERROR_DB,
        ),
        check(
            "p90_abs_error_db",
            evaluation.p90_abs_error_db,
            "<=",
            MAX_P90_ERROR_DB,
        ),
        check(
            "persistent_median_abs_error_db",
            evaluation.persistent_median_abs_error_db,
            "<=",
            MAX_PERSISTENT_MEDIAN_ERROR_DB,
        ),
        check(
            "improved_component_fraction",
            evaluation.improved_component_fraction,
            ">=",
            MIN_IMPROVED_COMPONENT_FRACTION,
        ),
        check(
            "median_error_ratio_to_constant",
            evaluation.median_error_ratio_to_constant,
            "<=",
            MAX_MEDIAN_RATIO_TO_CONSTANT,
        ),
    ];
    Gate {
        passed: checks.iter().all(|check| check.passed),
        checks,
    }
}

fn check(metric: &'static str, observed: f64, relation: &'static str, threshold: f64) -> GateCheck {
    let passed = match relation {
        ">=" => observed >= threshold,
        "<=" => observed <= threshold,
        _ => false,
    };
    GateCheck {
        metric,
        observed,
        relation,
        threshold,
        passed,
    }
}

fn select_candidate<'a>(
    development: &'a [CandidateEvaluation],
    calibration: &'a [CandidateEvaluation],
) -> Result<&'a CandidateEvaluation, String> {
    let eligible = development
        .iter()
        .filter(|result| result.gate.passed)
        .map(|result| result.candidate_id)
        .collect::<BTreeSet<_>>();
    calibration
        .iter()
        .filter(|result| eligible.contains(result.candidate_id) && result.gate.passed)
        .min_by(|left, right| {
            left.evaluation
                .calibration_loss
                .total_cmp(&right.evaluation.calibration_loss)
                .then_with(|| left.candidate_id.cmp(right.candidate_id))
        })
        .ok_or_else(|| "no preregistered candidate passed the Shell calibration gate".to_owned())
}

fn candidate_by_id(id: &str) -> Result<CandidateProfile, String> {
    CANDIDATES
        .iter()
        .copied()
        .find(|candidate| candidate.id == id)
        .ok_or_else(|| format!("selected spatial candidate is unknown: {id}"))
}

fn acquire_block(profile: &'static FrozenProfile) -> Result<AcquiredBlock, String> {
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
    let central_entries = parse_central_directory(profile, &central.body)?;
    validate_central_entries(profile, &central_entries)?;

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
    let audio_spec = entry(profile, profile.audio_entry_name)?;
    let compressed = fetch_entry(
        profile,
        &resolve,
        audio_spec,
        COMPRESSED_PREFIX_BYTES,
        &mut fetched_bytes,
    )?;
    let compressed_prefix_sha256 = sha256_hex(&compressed);
    let (payload, row_sha256, rows) = extract_block(profile, &compressed)?;
    validate_block_identities(profile, &raw_entries)?;
    Ok(AcquiredBlock {
        profile,
        compressed_prefix_sha256,
        payload_sha256: sha256_hex(&payload),
        payload,
        row_sha256,
        rows,
        fetched_bytes,
    })
}

type ExtractedBlock = (Vec<u8>, Vec<String>, Vec<Vec<f64>>);

fn extract_block(profile: &FrozenProfile, compressed: &[u8]) -> Result<ExtractedBlock, String> {
    let mut decoder = DeflateDecoder::new(compressed);
    let mut header = [0_u8; 128];
    decoder
        .read_exact(&mut header)
        .map_err(|error| format!("decompress REALIMPACT transfer NPY header: {error}"))?;
    validate_npy_header_prefix(&header, "<f4", &[3_000, profile.audio_sample_count])?;
    let mut payload = Vec::with_capacity(LISTENER_COUNT * profile.audio_sample_count * 4);
    let mut row_sha256 = Vec::new();
    let mut rows = Vec::new();
    for row_index in 0..LISTENER_COUNT {
        let mut bytes = vec![0_u8; profile.audio_sample_count * 4];
        decoder
            .read_exact(&mut bytes)
            .map_err(|error| format!("decompress REALIMPACT transfer row {row_index}: {error}"))?;
        let sha256 = sha256_hex(&bytes);
        if row_index == 0 {
            require_hash(
                &bytes,
                profile.audio_row_sha256,
                "REALIMPACT transfer row 0",
            )?;
        }
        rows.push(decode_row(&bytes, row_index)?);
        row_sha256.push(sha256);
        payload.extend_from_slice(&bytes);
    }
    Ok((payload, row_sha256, rows))
}

fn validate_block_identities(
    profile: &FrozenProfile,
    raw_entries: &BTreeMap<&'static str, Vec<u8>>,
) -> Result<(), String> {
    let vertex_xyz = f64_array(raw(raw_entries, "vertexXYZ.npy")?, &[3_000, 3])?;
    let listener_xyz = f64_array(raw(raw_entries, "listenerXYZ.npy")?, &[3_000, 3])?;
    let vertex_ids = i64_array(raw(raw_entries, "vertexID.npy")?, &[3_000])?;
    let microphones = i64_array(raw(raw_entries, "micID.npy")?, &[3_000])?;
    let distances = i64_array(raw(raw_entries, "distance.npy")?, &[3_000])?;
    let angles = i64_array(raw(raw_entries, "angle.npy")?, &[3_000])?;
    for row_index in 0..LISTENER_COUNT {
        let offset = row_index * 3;
        if vertex_ids[row_index] != profile.expected_impact_vertex_id as i64
            || vertex_xyz[offset..offset + 3] != profile.expected_impact_position
            || microphones[row_index] != row_index as i64
            || distances[row_index] != 0
            || angles[row_index] != 0
            || listener_xyz[offset..offset + 3] != [0.23, -0.04345, LISTENER_Z_METRES[row_index]]
        {
            return Err(format!(
                "REALIMPACT {} listener identity changed at row {row_index}",
                profile.dataset_object_id
            ));
        }
    }
    Ok(())
}

fn archive_range(profile: &'static FrozenProfile) -> ArchiveRange {
    ArchiveRange {
        archive_url: profile.archive_url.to_owned(),
        archive_bytes: profile.archive_bytes,
        central_directory_sha256: profile.central_sha256.to_owned(),
        audio_data_offset: entry(profile, profile.audio_entry_name)
            .expect("frozen audio entry")
            .data_offset,
        compressed_prefix_bytes: COMPRESSED_PREFIX_BYTES,
    }
}

fn frozen_gates() -> Gates {
    Gates {
        minimum_component_count: MIN_COMPONENT_COUNT,
        maximum_median_abs_error_db: MAX_MEDIAN_ERROR_DB,
        maximum_p90_abs_error_db: MAX_P90_ERROR_DB,
        maximum_persistent_median_abs_error_db: MAX_PERSISTENT_MEDIAN_ERROR_DB,
        minimum_improved_component_fraction: MIN_IMPROVED_COMPONENT_FRACTION,
        maximum_median_error_ratio_to_constant: MAX_MEDIAN_RATIO_TO_CONSTANT,
    }
}

#[allow(clippy::too_many_arguments)]
fn publish(
    output: &Path,
    preregistration: &[u8],
    transfer: &[u8],
    green_manifest: &[u8],
    selection: &[u8],
    shell: &AcquiredBlock,
    skull: &AcquiredBlock,
    report: &[u8],
) -> Result<(), String> {
    let parent = output
        .parent()
        .ok_or_else(|| "spatial output has no parent".to_owned())?;
    let sequence = NEXT_STAGING.fetch_add(1, Ordering::Relaxed);
    let staging = parent.join(format!(
        ".nextengine-realimpact-spatial-{}-{sequence}",
        std::process::id()
    ));
    fs::create_dir(&staging)
        .map_err(|error| format!("create spatial staging directory: {error}"))?;
    let guard = StagingGuard(staging.clone());
    write_file(
        &staging.join("preregistration-manifest.json"),
        preregistration,
    )?;
    write_file(&staging.join("transfer-calibration-report.json"), transfer)?;
    write_file(
        &staging.join("green-development-manifest.json"),
        green_manifest,
    )?;
    write_file(&staging.join("selection.json"), selection)?;
    write_file(&staging.join("shell-listener-block.f32le"), &shell.payload)?;
    write_file(&staging.join("skull-listener-block.f32le"), &skull.payload)?;
    write_file(&staging.join("report.json"), report)?;
    if output.exists() {
        fs::remove_dir(output)
            .map_err(|error| format!("remove confirmed-empty spatial output: {error}"))?;
    }
    fs::rename(&staging, output).map_err(|error| format!("publish spatial output: {error}"))?;
    std::mem::forget(guard);
    Ok(())
}

fn write_file(path: &Path, bytes: &[u8]) -> Result<(), String> {
    fs::write(path, bytes).map_err(|error| format!("write {}: {error}", path.display()))
}

struct StagingGuard(PathBuf);

impl Drop for StagingGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct AcquiredBlock {
    profile: &'static FrozenProfile,
    compressed_prefix_sha256: String,
    payload: Vec<u8>,
    payload_sha256: String,
    row_sha256: Vec<String>,
    rows: Vec<Vec<f64>>,
    fetched_bytes: u64,
}

impl AcquiredBlock {
    fn summary(&self) -> AcquisitionSummary<'_> {
        AcquisitionSummary {
            dataset_object_id: self.profile.dataset_object_id,
            archive_url: self.profile.archive_url,
            archive_content_length: self.profile.archive_bytes,
            central_directory_sha256: self.profile.central_sha256,
            compressed_prefix_bytes: COMPRESSED_PREFIX_BYTES,
            compressed_prefix_sha256: &self.compressed_prefix_sha256,
            http_range_payload_bytes: self.fetched_bytes,
            full_archive_fraction: self.fetched_bytes as f64 / self.profile.archive_bytes as f64,
            block_payload_sha256: &self.payload_sha256,
            row_sha256: &self.row_sha256,
        }
    }
}

#[derive(Debug, Deserialize)]
struct Manifest {
    schema: String,
    study_id: String,
    revision: String,
    transfer_calibration_report: FileRef,
    green_development_manifest: FileRef,
    green_development_block: FileRef,
    object_split: ObjectSplit,
    anchor_listeners: [usize; 9],
    held_listeners: [usize; 6],
    reference_listener: usize,
    window_samples: usize,
    onset_peak_fraction: f64,
    candidate_ids: Vec<String>,
    gates: Gates,
    shell_archive: ArchiveRange,
    skull_archive: ArchiveRange,
    prior_exposures: Vec<String>,
    allowed_claims: Vec<String>,
    prohibited_claims: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct FileRef {
    path: PathBuf,
    sha256: String,
}

#[derive(Debug, Deserialize)]
struct ObjectSplit {
    development: String,
    calibration: String,
    holdout: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
struct Gates {
    minimum_component_count: usize,
    maximum_median_abs_error_db: f64,
    maximum_p90_abs_error_db: f64,
    maximum_persistent_median_abs_error_db: f64,
    minimum_improved_component_fraction: f64,
    maximum_median_error_ratio_to_constant: f64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct ArchiveRange {
    archive_url: String,
    archive_bytes: u64,
    central_directory_sha256: String,
    audio_data_offset: u64,
    compressed_prefix_bytes: u64,
}

#[derive(Deserialize)]
struct TransferReport {
    schema: String,
    status: String,
    decision: String,
    selected_profile_id: String,
    selected_rows: Vec<TransferRow>,
    holdout_evaluation: TransferRow,
}

#[derive(Deserialize)]
struct TransferRow {
    profile_row_id: String,
    analysis: TransferAnalysis,
}

#[derive(Deserialize)]
struct TransferAnalysis {
    modes: Vec<TransferMode>,
}

#[derive(Deserialize)]
struct TransferMode {
    frequency_hz: f64,
    matched_tail_frequency_hz: Option<f64>,
}

#[derive(Deserialize)]
struct GreenManifest {
    schema: String,
    status: String,
    dataset_object_id: String,
    sample_rate_hz: u32,
    sample_count_per_row: usize,
    row_count: usize,
    block_payload: GreenPayload,
    rows: Vec<GreenRow>,
}

#[derive(Deserialize)]
struct GreenPayload {
    sha256: String,
}

#[derive(Deserialize)]
struct GreenRow {
    row_index: usize,
    microphone_id: usize,
    listener_position_metres: [f64; 3],
}

#[derive(Serialize)]
struct CandidateEvaluation {
    candidate_id: &'static str,
    gate: Gate,
    evaluation: Evaluation,
}

#[derive(Serialize)]
struct Gate {
    passed: bool,
    checks: Vec<GateCheck>,
}

#[derive(Serialize)]
struct GateCheck {
    metric: &'static str,
    observed: f64,
    relation: &'static str,
    threshold: f64,
    passed: bool,
}

#[derive(Serialize)]
struct SelectionSnapshot<'a> {
    schema: &'static str,
    preregistration_manifest_sha256: &'a str,
    selected_candidate_id: &'static str,
    selected_calibration_loss: f64,
    shell_calibration_gate_passed: bool,
    skull_holdout_opened: bool,
}

#[derive(Serialize)]
struct AcquisitionSummary<'a> {
    dataset_object_id: &'static str,
    archive_url: &'static str,
    archive_content_length: u64,
    central_directory_sha256: &'static str,
    compressed_prefix_bytes: u64,
    compressed_prefix_sha256: &'a str,
    http_range_payload_bytes: u64,
    full_archive_fraction: f64,
    block_payload_sha256: &'a str,
    row_sha256: &'a [String],
}

#[derive(Serialize)]
struct Report<'a> {
    schema: &'static str,
    status: &'static str,
    decision: &'static str,
    claim: &'static str,
    study_id: &'static str,
    revision: &'static str,
    preregistration_manifest_sha256: &'a str,
    transfer_calibration_report_sha256: &'static str,
    green_development_manifest_sha256: &'static str,
    green_development_block_sha256: &'static str,
    development: Vec<CandidateEvaluation>,
    shell_calibration: Vec<CandidateEvaluation>,
    selection_sha256: &'a str,
    selected_candidate_id: &'static str,
    skull_holdout: CandidateEvaluation,
    shell_acquisition: AcquisitionSummary<'a>,
    skull_acquisition: AcquisitionSummary<'a>,
    holdout_opened_after_selection_hash: bool,
    allowed_claims: &'static [&'static str],
    prohibited_claims: &'static [&'static str],
    remaining_spatial_scope: &'static str,
    next_action: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_and_gate_protocol_is_frozen() {
        assert_eq!(CANDIDATES.len(), 4);
        assert_eq!(ANCHOR_LISTENERS.len(), 9);
        assert_eq!(HELD_LISTENERS.len(), 6);
        assert_eq!(PREREGISTRATION_MANIFEST_SHA256.len(), 64);
        assert!(PROHIBITED_CLAIMS.contains(&"three_dimensional_spatial_field"));
        assert!(!ALLOWED_CLAIMS.contains(&"three_dimensional_spatial_field"));
    }

    #[test]
    fn selection_requires_development_and_calibration_gates() {
        let make = |id, loss, passed| CandidateEvaluation {
            candidate_id: id,
            gate: Gate {
                passed,
                checks: Vec::new(),
            },
            evaluation: Evaluation {
                component_count: 16,
                persistent_component_count: 8,
                held_listener_count: 6,
                median_abs_error_db: 1.0,
                p90_abs_error_db: 2.0,
                persistent_median_abs_error_db: 1.0,
                constant_median_abs_error_db: 4.0,
                median_error_ratio_to_constant: 0.25,
                improved_component_fraction: 1.0,
                calibration_loss: loss,
                components: Vec::new(),
            },
        };
        let development = vec![
            make(CANDIDATES[0].id, 9.0, true),
            make(CANDIDATES[1].id, 9.0, false),
        ];
        let calibration = vec![
            make(CANDIDATES[0].id, 2.0, true),
            make(CANDIDATES[1].id, 1.0, true),
        ];
        assert_eq!(
            select_candidate(&development, &calibration)
                .expect("selection")
                .candidate_id,
            CANDIDATES[0].id
        );
    }
}
