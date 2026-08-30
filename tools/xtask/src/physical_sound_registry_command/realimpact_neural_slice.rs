use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

use super::{
    FileRef, MAX_MANIFEST_BYTES, MAX_REFERENCED_FILE_BYTES, canonical_external_file,
    read_bounded_file, require_empty_output, resolve_artifact, resolve_cli_path,
    resolve_output_path, set_once, sha256_hex, validate_file_ref, validate_label,
};

const SOURCE_MANIFEST_SCHEMA: &str =
    "nextengine.experimental-realimpact-listener-block.manifest.v1";
const SOURCE_REPORT_SCHEMA: &str =
    "nextengine.experimental-realimpact-listener-block-acquisition.report.v1";
const SOURCE_REPORT_CLAIM: &str = "EXACT_ROW_LISTENER_AND_SAME_IMPACT_BLOCK_IDENTITY_ONLY / NO_SPATIAL_MODEL_VALIDATION_ADMISSION_PASS_OR_RUNTIME_AUTHORITY";
const SLICE_SCHEMA: &str = "nextengine.experimental-realimpact-neural-slice.v1";
const REPORT_SCHEMA: &str = "nextengine.experimental-realimpact-neural-slice.report.v1";
const REPORT_CLAIM: &str = "RELATIVE_MULTI_LISTENER_FORCE_DECONVOLVED_TRANSFER_WAV_SLICE_ONLY / NO_ABSOLUTE_AMPLITUDE_MODEL_QUALITY_ADMISSION_OR_RUNTIME_AUTHORITY";
const MAX_ROWS: usize = 4_096;

pub(super) struct Request {
    listener_manifest: PathBuf,
    acquisition_report: PathBuf,
    output: PathBuf,
}

pub(super) fn run_cli(root: &Path, arguments: impl Iterator<Item = String>) -> Result<(), String> {
    let request = parse_arguments(arguments)?;
    run(root, &request)
}

fn parse_arguments(mut arguments: impl Iterator<Item = String>) -> Result<Request, String> {
    let mut listener_manifest = None;
    let mut acquisition_report = None;
    let mut output = None;
    while let Some(flag) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--listener-manifest" => set_once(&mut listener_manifest, PathBuf::from(value), &flag)?,
            "--acquisition-report" => {
                set_once(&mut acquisition_report, PathBuf::from(value), &flag)?
            }
            "--output" => set_once(&mut output, PathBuf::from(value), &flag)?,
            _ => {
                return Err(format!(
                    "unexpected realimpact-neural-slice argument: {flag}"
                ));
            }
        }
    }
    Ok(Request {
        listener_manifest: listener_manifest.ok_or_else(|| {
            "realimpact-neural-slice requires --listener-manifest <external-json>".to_owned()
        })?,
        acquisition_report: acquisition_report.ok_or_else(|| {
            "realimpact-neural-slice requires --acquisition-report <external-json>".to_owned()
        })?,
        output: output.ok_or_else(|| {
            "realimpact-neural-slice requires --output <external-empty-directory>".to_owned()
        })?,
    })
}

#[derive(Deserialize)]
struct ListenerBlockManifest {
    schema: String,
    status: String,
    profile: String,
    source_repository_revision: String,
    dataset_object_id: String,
    object_id: String,
    geometry_revision: String,
    impact_position_id: String,
    impact_position_metres: [f64; 3],
    sample_rate_hz: u32,
    sample_count_per_row: usize,
    row_count: usize,
    block_payload: SourceArtifact,
    rows: Vec<SourceRow>,
}

#[derive(Deserialize)]
struct SourceArtifact {
    path: String,
    sha256: String,
    byte_count: usize,
}

#[derive(Deserialize)]
struct SourceRow {
    row_index: usize,
    microphone_id: usize,
    listener_condition_id: String,
    listener_position_metres: [f64; 3],
    payload_offset_bytes: usize,
    sample_count: usize,
    raw_f32le_sha256: String,
    peak_abs: f64,
}

#[derive(Deserialize)]
struct AcquisitionReport {
    schema: String,
    status: String,
    claim: String,
    profile: String,
    manifest_sha256: String,
    row_count: usize,
    same_impact_vertex: bool,
}

#[derive(Serialize)]
struct NeuralSlice {
    schema: &'static str,
    status: &'static str,
    claim: &'static str,
    profile: String,
    source_repository_revision: String,
    source_manifest_sha256: String,
    source_acquisition_report_sha256: String,
    source_block_sha256: String,
    dataset_object_id: String,
    object_id: String,
    geometry_revision: String,
    impact_position_id: String,
    impact_position_metres: [f64; 3],
    impact_outward_normal: Option<[f64; 3]>,
    sample_rate_hz: u32,
    audio_semantics: &'static str,
    normalization: Normalization,
    rows: Vec<SliceRow>,
}

#[derive(Serialize)]
struct Normalization {
    policy: &'static str,
    shared_raw_peak_abs: f64,
    target_peak_fraction_of_i16_max: f64,
    relative_amplitude_preserved: bool,
    absolute_amplitude_credit: bool,
}

#[derive(Serialize)]
struct SliceRow {
    row_index: usize,
    microphone_id: usize,
    listener_condition_id: String,
    listener_position_metres: [f64; 3],
    source_raw_f32le_sha256: String,
    wav_file: String,
    wav_sha256: String,
    wav_byte_count: usize,
}

#[derive(Serialize)]
struct SliceReport {
    schema: &'static str,
    status: &'static str,
    decision: &'static str,
    claim: &'static str,
    profile: String,
    source_manifest_sha256: String,
    source_acquisition_report_sha256: String,
    slice_sha256: String,
    row_count: usize,
    shared_raw_peak_abs: f64,
    audio_semantics: &'static str,
    relative_amplitude_preserved: bool,
    model_training_authorized: bool,
    quality_or_admission_authorized: bool,
    emitted_files: Vec<String>,
}

fn run(root: &Path, request: &Request) -> Result<(), String> {
    let root =
        fs::canonicalize(root).map_err(|error| format!("canonicalize repository root: {error}"))?;
    let manifest_path = canonical_external_file(
        &root,
        &resolve_cli_path(&root, &request.listener_manifest),
        "REALIMPACT listener-block manifest",
    )?;
    let report_path = canonical_external_file(
        &root,
        &resolve_cli_path(&root, &request.acquisition_report),
        "REALIMPACT listener-block acquisition report",
    )?;
    let output = resolve_output_path(&root, &request.output)?;
    require_empty_output(&output)?;

    let manifest_bytes = read_bounded_file(
        &manifest_path,
        MAX_MANIFEST_BYTES,
        "REALIMPACT listener-block manifest",
    )?;
    let report_bytes = read_bounded_file(
        &report_path,
        MAX_MANIFEST_BYTES,
        "REALIMPACT listener-block acquisition report",
    )?;
    let manifest: ListenerBlockManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("parse {}: {error}", manifest_path.display()))?;
    let report: AcquisitionReport = serde_json::from_slice(&report_bytes)
        .map_err(|error| format!("parse {}: {error}", report_path.display()))?;
    let manifest_sha256 = sha256_hex(&manifest_bytes);
    let report_sha256 = sha256_hex(&report_bytes);
    validate_source(&manifest, &report, &manifest_sha256)?;

    let manifest_directory = manifest_path
        .parent()
        .ok_or_else(|| "REALIMPACT listener manifest has no parent directory".to_owned())?;
    let block_ref = FileRef {
        path: manifest.block_payload.path.clone(),
        sha256: manifest.block_payload.sha256.clone(),
    };
    validate_file_ref(&block_ref, "REALIMPACT listener block")?;
    let block_summary = resolve_artifact(
        &root,
        manifest_directory,
        &block_ref,
        "REALIMPACT listener block",
    )?;
    if block_summary.byte_count != manifest.block_payload.byte_count {
        return Err("REALIMPACT listener-block byte count changed".to_owned());
    }
    let block_path = canonical_external_file(
        &root,
        &manifest_directory.join(&manifest.block_payload.path),
        "REALIMPACT listener block",
    )?;
    let block = read_bounded_file(
        &block_path,
        MAX_REFERENCED_FILE_BYTES,
        "REALIMPACT listener block",
    )?;
    let shared_peak = validate_rows_and_peak(&manifest, &block)?;

    let mut wavs = Vec::with_capacity(manifest.rows.len());
    let mut rows = Vec::with_capacity(manifest.rows.len());
    let mut emitted_files = vec!["report.json".to_owned(), "slice.json".to_owned()];
    for row in &manifest.rows {
        let row_bytes = row_bytes(row, &block)?;
        let wav = super::realimpact_row::normalized_wav(row_bytes, shared_peak)?;
        let wav_file = format!(
            "row-{:04}-mic-{:02}.relative-transfer.wav",
            row.row_index, row.microphone_id
        );
        let wav_sha256 = sha256_hex(&wav);
        emitted_files.push(wav_file.clone());
        rows.push(SliceRow {
            row_index: row.row_index,
            microphone_id: row.microphone_id,
            listener_condition_id: row.listener_condition_id.clone(),
            listener_position_metres: row.listener_position_metres,
            source_raw_f32le_sha256: row.raw_f32le_sha256.clone(),
            wav_file: wav_file.clone(),
            wav_sha256,
            wav_byte_count: wav.len(),
        });
        wavs.push((wav_file, wav));
    }

    let slice = NeuralSlice {
        schema: SLICE_SCHEMA,
        status: "Validated",
        claim: REPORT_CLAIM,
        profile: manifest.profile.clone(),
        source_repository_revision: manifest.source_repository_revision,
        source_manifest_sha256: manifest_sha256.clone(),
        source_acquisition_report_sha256: report_sha256.clone(),
        source_block_sha256: manifest.block_payload.sha256,
        dataset_object_id: manifest.dataset_object_id,
        object_id: manifest.object_id,
        geometry_revision: manifest.geometry_revision,
        impact_position_id: manifest.impact_position_id,
        impact_position_metres: manifest.impact_position_metres,
        impact_outward_normal: None,
        sample_rate_hz: manifest.sample_rate_hz,
        audio_semantics: "force_deconvolved_transfer_response",
        normalization: Normalization {
            policy: "one_shared_block_peak_to_0.95_i16_v1",
            shared_raw_peak_abs: shared_peak,
            target_peak_fraction_of_i16_max: 0.95,
            relative_amplitude_preserved: true,
            absolute_amplitude_credit: false,
        },
        rows,
    };
    let slice_bytes = serde_json::to_vec_pretty(&slice)
        .map_err(|error| format!("serialize REALIMPACT neural slice: {error}"))?;
    let slice_sha256 = sha256_hex(&slice_bytes);
    emitted_files.sort();
    let report = SliceReport {
        schema: REPORT_SCHEMA,
        status: "Validated",
        decision: "RelativeTransferSliceAvailable",
        claim: REPORT_CLAIM,
        profile: manifest.profile,
        source_manifest_sha256: manifest_sha256,
        source_acquisition_report_sha256: report_sha256,
        slice_sha256,
        row_count: slice.rows.len(),
        shared_raw_peak_abs: shared_peak,
        audio_semantics: "force_deconvolved_transfer_response",
        relative_amplitude_preserved: true,
        model_training_authorized: false,
        quality_or_admission_authorized: false,
        emitted_files,
    };
    let report_bytes = serde_json::to_vec_pretty(&report)
        .map_err(|error| format!("serialize REALIMPACT neural-slice report: {error}"))?;

    fs::create_dir_all(&output).map_err(|error| format!("create {}: {error}", output.display()))?;
    for (file, wav) in wavs {
        fs::write(output.join(&file), wav).map_err(|error| format!("write {file}: {error}"))?;
    }
    fs::write(output.join("slice.json"), slice_bytes)
        .map_err(|error| format!("write slice.json: {error}"))?;
    fs::write(output.join("report.json"), &report_bytes)
        .map_err(|error| format!("write report.json: {error}"))?;
    println!(
        "{}",
        String::from_utf8(report_bytes).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn validate_source(
    manifest: &ListenerBlockManifest,
    report: &AcquisitionReport,
    manifest_sha256: &str,
) -> Result<(), String> {
    if manifest.schema != SOURCE_MANIFEST_SCHEMA
        || manifest.status != "development_pilot_partial_source"
    {
        return Err("unsupported REALIMPACT listener-block manifest".to_owned());
    }
    if report.schema != SOURCE_REPORT_SCHEMA
        || report.status != "Validated"
        || report.claim != SOURCE_REPORT_CLAIM
    {
        return Err("unsupported REALIMPACT listener-block acquisition report".to_owned());
    }
    validate_label(&manifest.profile, "REALIMPACT listener profile")?;
    validate_label(&manifest.object_id, "REALIMPACT listener object id")?;
    validate_label(&manifest.geometry_revision, "REALIMPACT geometry revision")?;
    validate_label(
        &manifest.impact_position_id,
        "REALIMPACT impact position id",
    )?;
    if report.profile != manifest.profile
        || report.manifest_sha256 != manifest_sha256
        || report.row_count != manifest.row_count
        || !report.same_impact_vertex
    {
        return Err("REALIMPACT listener manifest/report lineage mismatch".to_owned());
    }
    if manifest.sample_rate_hz != 48_000
        || manifest.sample_count_per_row == 0
        || manifest.row_count < 2
        || manifest.row_count > MAX_ROWS
        || manifest.rows.len() != manifest.row_count
    {
        return Err("REALIMPACT listener-block dimensions are invalid".to_owned());
    }
    if manifest
        .impact_position_metres
        .into_iter()
        .any(|value| !value.is_finite())
    {
        return Err("REALIMPACT impact position is non-finite".to_owned());
    }
    Ok(())
}

fn validate_rows_and_peak(manifest: &ListenerBlockManifest, block: &[u8]) -> Result<f64, String> {
    let expected_row_bytes = manifest
        .sample_count_per_row
        .checked_mul(4)
        .ok_or_else(|| "REALIMPACT row byte count overflow".to_owned())?;
    let expected_block_bytes = expected_row_bytes
        .checked_mul(manifest.row_count)
        .ok_or_else(|| "REALIMPACT block byte count overflow".to_owned())?;
    if block.len() != expected_block_bytes {
        return Err("REALIMPACT listener block dimensions changed".to_owned());
    }
    let mut shared_peak = 0.0_f64;
    for (expected_index, row) in manifest.rows.iter().enumerate() {
        if row.row_index != expected_index
            || row.sample_count != manifest.sample_count_per_row
            || row.payload_offset_bytes != expected_index * expected_row_bytes
            || !row.peak_abs.is_finite()
            || row.peak_abs <= 0.0
        {
            return Err(format!(
                "REALIMPACT listener row {} shape or order changed",
                row.row_index
            ));
        }
        validate_label(
            &row.listener_condition_id,
            "REALIMPACT listener condition id",
        )?;
        if row
            .listener_position_metres
            .into_iter()
            .any(|value| !value.is_finite())
        {
            return Err(format!(
                "REALIMPACT listener row {} position is non-finite",
                row.row_index
            ));
        }
        let bytes = row_bytes(row, block)?;
        if sha256_hex(bytes) != row.raw_f32le_sha256 {
            return Err(format!(
                "REALIMPACT listener row {} hash changed",
                row.row_index
            ));
        }
        let mut row_peak = 0.0_f64;
        for sample in bytes.chunks_exact(4) {
            let sample = f64::from(f32::from_le_bytes(
                sample.try_into().expect("four-byte REALIMPACT sample"),
            ));
            if !sample.is_finite() {
                return Err(format!(
                    "REALIMPACT listener row {} contains a non-finite sample",
                    row.row_index
                ));
            }
            row_peak = row_peak.max(sample.abs());
        }
        if row_peak != row.peak_abs {
            return Err(format!(
                "REALIMPACT listener row {} peak changed",
                row.row_index
            ));
        }
        shared_peak = shared_peak.max(row_peak);
    }
    if shared_peak <= 0.0 {
        return Err("REALIMPACT listener block is silent".to_owned());
    }
    Ok(shared_peak)
}

fn row_bytes<'a>(row: &SourceRow, block: &'a [u8]) -> Result<&'a [u8], String> {
    let byte_count = row
        .sample_count
        .checked_mul(4)
        .ok_or_else(|| "REALIMPACT row byte count overflow".to_owned())?;
    let end = row
        .payload_offset_bytes
        .checked_add(byte_count)
        .ok_or_else(|| "REALIMPACT row offset overflow".to_owned())?;
    block.get(row.payload_offset_bytes..end).ok_or_else(|| {
        format!(
            "REALIMPACT listener row {} exceeds block payload",
            row.row_index
        )
    })
}
