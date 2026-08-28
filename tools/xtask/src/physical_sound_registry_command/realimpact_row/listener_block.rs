use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use flate2::read::DeflateDecoder;
use serde::{Deserialize, Serialize};

use super::evidence::{pretty_json, read_source_bundle};
use super::profiles::{GREEN_GOBLET_PROFILE_ID, frozen_profile};
use super::*;

pub(super) const PROFILE_ID: &str = "green-goblet-listener-block-0-v1";

const TRANSFER_REPORT_SHA256: &str =
    "01346767b596630061fe437e98d5213bf426e49e5b96c22565a77acfea50d444";
const TRANSFER_REPORT_SCHEMA: &str =
    "nextengine.experimental-realimpact-transfer-calibration.report.v2";
const COMPRESSED_PREFIX_BYTES: u64 = 16 * 1024 * 1024;
const COMPRESSED_PREFIX_SHA256: &str =
    "582ff65d41d7425886392694304e2fdb5f9e1b21cf4a6e289ed97481cf4049a0";
const BLOCK_ROW_COUNT: usize = 15;
const BLOCK_PAYLOAD_SHA256: &str =
    "8bcffd0a9f57fd101a803228f7e8aa66d469f82d99ad6e43950318aae0f875ca";
const BLOCK_FILE_NAME: &str = "green-goblet-listener-block-0000-0014.f32le";
const MANIFEST_SCHEMA: &str = "nextengine.experimental-realimpact-listener-block.manifest.v1";
const REPORT_SCHEMA: &str =
    "nextengine.experimental-realimpact-listener-block-acquisition.report.v1";

const ROW_SHA256: [&str; BLOCK_ROW_COUNT] = [
    "104dd97391bf6319ccbd4dfdf48569be58097bf1f90cf8cea3ae3cb2f7498ec9",
    "f8e0c4c1d8dbf8a8cd42d43552fdff34488a7e2fe5a70cf15e3f6db5be9ee487",
    "a27d40bd7c36049e647da43f8c50b49acbb4855b97deca5168d0d03445224f10",
    "e356b3af261aeaa14b3fce7a7b40ca81d7c1256ab14234a81ae6de679c188430",
    "f559f4f5bbbcb18f92a0229cfa1643c20e7b887acc999069135192ccadd25c42",
    "d5e008cf51d70f75101e56b2f3b88862a9a0d2be34ed5db6a7a376e7ff5325d9",
    "658cf8e37f8423997e8ec5f3569ac5ed9abf4f9fd2dcc3960a9100f1859515a1",
    "cd6f175a00fb5ea1f5c7e66f07e8a3f1ce0493bf2db5c5708b3e141082da180d",
    "43721c43cf6d86f42445ee5f221b819284ea7a5c8d7403131b826c8bda89fe4f",
    "ee51ca54e5e01c6d7e0fab474adc31457587afd11a76a43921f2fa07f5dedfdb",
    "a1aae21b7a7b85aef9853384cc7272475efb7eab8ca9eb642a54859a9af96008",
    "02e61a41868618f66584cdc4b206f0f463dda696d95d98def20957782af15d0e",
    "43a352b826e63e347a547d2ef01dfeeb0b328bf37fdced11ba659c59a27ae460",
    "7efb1c3b756dc5db19c0c8017645aaf8f6f0d87df19b972f78b17ff922e0806d",
    "e40fa7795e81c472d3cf06e4496542aeca4426173faabef6471e7bdc411f7438",
];

const LISTENER_Z_METRES: [f64; BLOCK_ROW_COUNT] = [
    -0.91, -0.78, -0.65, -0.52, -0.39, -0.26, -0.13, 0.0, 0.13, 0.26, 0.39, 0.52, 0.65, 0.78, 0.91,
];

const ALLOWED_CLAIMS: [&str; 3] = [
    "exact_row_to_microphone_identity",
    "same_published_impact_angle_distance_block",
    "development_relative_multi_listener_response",
];

const PROHIBITED_CLAIMS: [&str; 9] = [
    "absolute_amplitude_claim",
    "cross_object_spatial_generalization",
    "exact_material_composition_claim",
    "exact_support_fixture_claim",
    "matched_cross_tier_condition_claim",
    "physical_sound_pass",
    "production_corpus_admission",
    "runtime_content_role",
    "spatial_model_validation",
];

static NEXT_STAGING: AtomicU64 = AtomicU64::new(0);

pub(super) fn run(root: &Path, request: &Request) -> Result<(), String> {
    let output = resolve_output_path(root, &request.output)?;
    require_empty_output(&output)?;
    let transfer_path = request
        .transfer_calibration_report
        .as_ref()
        .ok_or_else(|| {
            "listener-block profile requires --transfer-calibration-report <external-json>"
                .to_owned()
        })?;
    let transfer_path = canonical_external_file(
        root,
        &resolve_cli_path(root, transfer_path),
        "REALIMPACT transfer-calibration report",
    )?;
    let transfer_report = read_bounded_file(
        &transfer_path,
        16 * 1024 * 1024,
        "REALIMPACT transfer-calibration report",
    )?;
    validate_transfer_report(&transfer_report)?;

    let source_bundle = resolve_cli_path(root, &request.source_bundle);
    let source_files = read_source_bundle(root, &source_bundle)?;
    let corpus_plan_path = canonical_external_file(
        root,
        &resolve_cli_path(root, &request.corpus_plan_report),
        "REALIMPACT corpus-plan report",
    )?;
    let corpus_plan = read_bounded_file(
        &corpus_plan_path,
        16 * 1024 * 1024,
        "REALIMPACT corpus-plan report",
    )?;
    require_hash(&corpus_plan, CORPUS_PLAN_SHA256, "corpus-plan report")?;

    let profile = frozen_profile(GREEN_GOBLET_PROFILE_ID)?;
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
    let audio_prefix = fetch_entry(
        profile,
        &resolve,
        audio_spec,
        COMPRESSED_PREFIX_BYTES,
        &mut fetched_bytes,
    )?;
    require_hash(
        &audio_prefix,
        COMPRESSED_PREFIX_SHA256,
        "Green Goblet listener-block compressed prefix",
    )?;
    let rows = extract_rows(profile, &audio_prefix)?;
    let row_zero = rows
        .first()
        .ok_or_else(|| "listener block has no row zero".to_owned())?;
    let derived = validate_and_derive(profile, &raw_entries, row_zero)?;
    let identities = validate_block_identities(profile, &raw_entries)?;
    let block_payload = rows
        .iter()
        .flat_map(|row| row.bytes.iter().copied())
        .collect::<Vec<_>>();
    require_hash(
        &block_payload,
        BLOCK_PAYLOAD_SHA256,
        "Green Goblet listener-block payload",
    )?;

    let frozen_profile = frozen_block_profile();
    let frozen_profile_bytes = pretty_json(&frozen_profile)?;
    let frozen_profile_sha256 = sha256_hex(&frozen_profile_bytes);
    let manifest_rows = rows
        .iter()
        .zip(identities)
        .map(|(row, identity)| ListenerRow {
            row_index: identity.row_index,
            microphone_id: identity.microphone_id,
            listener_condition_id: format!(
                "angle-000-distance-0230mm-mic-{:02}",
                identity.microphone_id
            ),
            listener_position_metres: identity.listener_position_metres,
            payload_offset_bytes: identity.row_index * profile.audio_sample_count * 4,
            sample_count: row.sample_count,
            raw_f32le_sha256: row.sha256.clone(),
            peak_abs: row.peak_abs,
            rms: row.rms,
        })
        .collect::<Vec<_>>();
    let manifest = ListenerBlockManifest {
        schema: MANIFEST_SCHEMA,
        status: "development_pilot_partial_source",
        profile: PROFILE_ID,
        frozen_profile_sha256: &frozen_profile_sha256,
        source_repository_revision: REPOSITORY_REVISION,
        dataset_object_id: profile.dataset_object_id,
        object_id: profile.object_id,
        geometry_revision: profile.geometry_revision,
        impact_position_id: profile.impact_position_id,
        impact_position_metres: derived.impact_position,
        azimuth_degrees: 0,
        gantry_distance_offset_millimetres: 0,
        listener_x_metres: 0.23,
        listener_y_metres: -0.04345,
        sample_rate_hz: 48_000,
        sample_count_per_row: profile.audio_sample_count,
        row_count: rows.len(),
        block_payload: FileIdentity {
            path: BLOCK_FILE_NAME,
            sha256: BLOCK_PAYLOAD_SHA256,
            byte_count: block_payload.len(),
        },
        rows: manifest_rows,
        metadata_array_sha256: derived.downloaded_hashes,
        prerequisites: Prerequisites {
            corpus_plan_report_sha256: CORPUS_PLAN_SHA256,
            transfer_calibration_report_sha256: TRANSFER_REPORT_SHA256,
            preprocess_measurements_sha256: SOURCE_FILES[2].sha256,
            preprocess_annotations_sha256: SOURCE_FILES[3].sha256,
        },
        row_order_proof: [
            "preprocess_measurements appends microphone indices 0..14 inside each valid impact-condition row",
            "preprocess_annotations writes angle, distance and microphone arrays in the same nested-loop order",
            "published arrays prove rows 0..14 share vertex 31676, angle 0 and distance offset 0 while microphone IDs are 0..14",
        ],
        allowed_claims: &ALLOWED_CLAIMS,
        prohibited_claims: &PROHIBITED_CLAIMS,
        unavailable_components: &UNAVAILABLE_COMPONENTS,
    };
    let manifest_bytes = pretty_json(&manifest)?;
    let manifest_sha256 = sha256_hex(&manifest_bytes);
    let provenance = provenance_review();
    let provenance_sha256 = sha256_hex(provenance.as_bytes());
    let report = ListenerBlockReport {
        schema: REPORT_SCHEMA,
        status: "Validated",
        decision: "MultiListenerAcquisitionPilotOnly",
        claim: "EXACT_ROW_LISTENER_AND_SAME_IMPACT_BLOCK_IDENTITY_ONLY / NO_SPATIAL_MODEL_VALIDATION_ADMISSION_PASS_OR_RUNTIME_AUTHORITY",
        profile: PROFILE_ID,
        frozen_profile_sha256: &frozen_profile_sha256,
        manifest_sha256: &manifest_sha256,
        provenance_sha256: &provenance_sha256,
        transfer_calibration_report_sha256: TRANSFER_REPORT_SHA256,
        archive_url: profile.archive_url,
        archive_content_length: profile.archive_bytes,
        central_directory_sha256: profile.central_sha256,
        compressed_prefix_bytes: COMPRESSED_PREFIX_BYTES,
        compressed_prefix_sha256: COMPRESSED_PREFIX_SHA256,
        http_range_payload_bytes: fetched_bytes,
        full_archive_fraction: fetched_bytes as f64 / profile.archive_bytes as f64,
        row_count: rows.len(),
        distinct_microphone_count: rows.len(),
        same_impact_vertex: true,
        same_azimuth: true,
        same_distance_offset: true,
        spatial_evaluation: "NotRunObjectDisjointRowsMissing",
        allowed_claims: &ALLOWED_CLAIMS,
        prohibited_claims: &PROHIBITED_CLAIMS,
        next_action: "while Shell Plate and Skull Cup listener rows 1..14 remain unopened, preregister the exact object split, candidates, metrics, gates and archive ranges; then use Shell Plate for calibration and Skull Cup once as the spatial-response holdout",
    };
    let report_bytes = pretty_json(&report)?;
    publish(
        &output,
        &source_files,
        &corpus_plan,
        &transfer_report,
        &block_payload,
        &frozen_profile_bytes,
        &manifest_bytes,
        provenance.as_bytes(),
        &report_bytes,
    )?;
    println!("REALIMPACT listener block output: {}", output.display());
    println!("block payload sha256: {BLOCK_PAYLOAD_SHA256}");
    println!("manifest sha256: {manifest_sha256}");
    println!("report decision: MultiListenerAcquisitionPilotOnly");
    Ok(())
}

fn validate_transfer_report(bytes: &[u8]) -> Result<(), String> {
    require_hash(bytes, TRANSFER_REPORT_SHA256, "transfer-calibration report")?;
    let report: TransferCalibrationReport = serde_json::from_slice(bytes)
        .map_err(|error| format!("parse transfer-calibration report: {error}"))?;
    validate_transfer_report_semantics(&report)
}

fn validate_transfer_report_semantics(report: &TransferCalibrationReport) -> Result<(), String> {
    if report.schema != TRANSFER_REPORT_SCHEMA
        || report.status != "Validated"
        || report.decision != "RelativeModalDampingSupportedSpatialUnavailable"
        || report.revision != "v2"
        || report.selected_profile_id != "injective-modal-16-fft65536-v2"
        || !report.holdout_gate.passed
        || report.spatial_participation.decision != "NotEvaluableSingleListenerRowPerObject"
        || report.spatial_participation.observed_rows_per_object != 1
        || report.spatial_participation.required_rows_per_object != 2
    {
        return Err(
            "transfer-calibration report does not authorize listener acquisition".to_owned(),
        );
    }
    Ok(())
}

fn extract_rows(profile: &FrozenProfile, compressed: &[u8]) -> Result<Vec<AudioRow>, String> {
    let mut decoder = DeflateDecoder::new(compressed);
    let mut header = [0_u8; 128];
    decoder
        .read_exact(&mut header)
        .map_err(|error| format!("decompress REALIMPACT transfer NPY header: {error}"))?;
    validate_npy_header_prefix(&header, "<f4", &[3_000, profile.audio_sample_count])?;
    (0..BLOCK_ROW_COUNT)
        .map(|row_index| {
            let mut bytes = vec![0_u8; profile.audio_sample_count * 4];
            decoder.read_exact(&mut bytes).map_err(|error| {
                format!("decompress REALIMPACT transfer row {row_index}: {error}")
            })?;
            require_hash(
                &bytes,
                ROW_SHA256[row_index],
                &format!("REALIMPACT transfer row {row_index}"),
            )?;
            let mut peak_abs = 0.0_f64;
            let mut sum_squared = 0.0_f64;
            for sample in bytes.chunks_exact(4) {
                let value = f64::from(f32::from_le_bytes(sample.try_into().expect("four bytes")));
                if !value.is_finite() {
                    return Err(format!(
                        "REALIMPACT transfer row {row_index} contains a non-finite sample"
                    ));
                }
                peak_abs = peak_abs.max(value.abs());
                sum_squared += value * value;
            }
            Ok(AudioRow {
                sha256: sha256_hex(&bytes),
                bytes,
                sample_count: profile.audio_sample_count,
                peak_abs,
                rms: (sum_squared / profile.audio_sample_count as f64).sqrt(),
            })
        })
        .collect()
}

fn validate_block_identities(
    profile: &FrozenProfile,
    raw_entries: &BTreeMap<&'static str, Vec<u8>>,
) -> Result<Vec<RowIdentity>, String> {
    let vertex_xyz = f64_array(raw(raw_entries, "vertexXYZ.npy")?, &[3_000, 3])?;
    let listener_xyz = f64_array(raw(raw_entries, "listenerXYZ.npy")?, &[3_000, 3])?;
    let vertex_ids = i64_array(raw(raw_entries, "vertexID.npy")?, &[3_000])?;
    let microphone_ids = i64_array(raw(raw_entries, "micID.npy")?, &[3_000])?;
    let distances = i64_array(raw(raw_entries, "distance.npy")?, &[3_000])?;
    let angles = i64_array(raw(raw_entries, "angle.npy")?, &[3_000])?;
    let mut positions = BTreeSet::new();
    let mut identities = Vec::new();
    for row_index in 0..BLOCK_ROW_COUNT {
        let offset = row_index * 3;
        let impact = [
            vertex_xyz[offset],
            vertex_xyz[offset + 1],
            vertex_xyz[offset + 2],
        ];
        let listener = [
            listener_xyz[offset],
            listener_xyz[offset + 1],
            listener_xyz[offset + 2],
        ];
        let expected_listener = [0.23, -0.04345, LISTENER_Z_METRES[row_index]];
        if vertex_ids[row_index] != profile.expected_impact_vertex_id as i64
            || impact != profile.expected_impact_position
            || angles[row_index] != 0
            || distances[row_index] != 0
            || microphone_ids[row_index] != row_index as i64
            || listener != expected_listener
            || !positions.insert(listener.map(f64::to_bits))
        {
            return Err(format!(
                "REALIMPACT Green Goblet listener identity changed at row {row_index}"
            ));
        }
        identities.push(RowIdentity {
            row_index,
            microphone_id: row_index,
            listener_position_metres: listener,
        });
    }
    Ok(identities)
}

fn frozen_block_profile() -> FrozenBlockProfile {
    FrozenBlockProfile {
        profile: PROFILE_ID,
        dataset_object_id: "93_GreenGoblet",
        source_row_range: [0, 14],
        row_count: BLOCK_ROW_COUNT,
        impact_vertex_id: 31_676,
        azimuth_degrees: 0,
        gantry_distance_offset_millimetres: 0,
        microphone_ids: (0..BLOCK_ROW_COUNT).collect(),
        listener_z_metres: LISTENER_Z_METRES,
        compressed_prefix_bytes: COMPRESSED_PREFIX_BYTES,
        compressed_prefix_sha256: COMPRESSED_PREFIX_SHA256,
        block_payload_sha256: BLOCK_PAYLOAD_SHA256,
        row_sha256: ROW_SHA256,
        allowed_claims: ALLOWED_CLAIMS,
        prohibited_claims: PROHIBITED_CLAIMS,
    }
}

fn provenance_review() -> String {
    "# REALIMPACT Green Goblet multi-listener acquisition provenance\n\n\
Status: development-only external E2 acquisition pilot; no spatial-model,\n\
corpus-admission, quality or runtime authority.\n\n\
Primary sources:\n\n\
- https://samuelpclarke.com/realimpact/\n\
- https://github.com/samuel-clarke/RealImpact\n\
- https://jiajunwu.com/papers/realimpact_cvpr.pdf\n\
- https://downloads.cs.stanford.edu/viscam/RealImpact/93_GreenGoblet.zip\n\n\
The frozen preprocessing scripts establish row order: inside each valid impact\n\
condition, microphone recordings 1 through 15 are appended in order, and the\n\
annotation arrays repeat the same nested loop. The published arrays independently\n\
prove that rows 0 through 14 share vertex 31676, azimuth 0 and distance offset 0,\n\
while microphone IDs and listener heights span 0 through 14 and -0.91 through\n\
0.91 metres. This is exact row/listener identity for one impact block only.\n\n\
The command reads a fixed 16 MiB compressed prefix of the 2.31 GB archive member\n\
and emits the first 15 deconvolved float32 rows as one external block. Raw force,\n\
material revision, repeat identity and versioned support fixture remain unavailable.\n\
The large recording bytes, reports and derived block stay outside Git.\n"
        .to_owned()
}

#[allow(clippy::too_many_arguments)]
fn publish(
    output: &Path,
    source_files: &[(&str, Vec<u8>)],
    corpus_plan: &[u8],
    transfer_report: &[u8],
    block_payload: &[u8],
    frozen_profile: &[u8],
    manifest: &[u8],
    provenance: &[u8],
    report: &[u8],
) -> Result<(), String> {
    let parent = output
        .parent()
        .ok_or_else(|| "listener-block output has no parent".to_owned())?;
    let sequence = NEXT_STAGING.fetch_add(1, Ordering::Relaxed);
    let staging = parent.join(format!(
        ".nextengine-realimpact-listener-block-{}-{sequence}",
        std::process::id()
    ));
    fs::create_dir(&staging)
        .map_err(|error| format!("create listener-block staging directory: {error}"))?;
    let guard = StagingGuard(staging.clone());
    write_file(&staging.join("corpus-plan-report.json"), corpus_plan)?;
    write_file(
        &staging.join("transfer-calibration-report.json"),
        transfer_report,
    )?;
    write_file(&staging.join(BLOCK_FILE_NAME), block_payload)?;
    write_file(&staging.join("frozen-profile.json"), frozen_profile)?;
    write_file(&staging.join("manifest.json"), manifest)?;
    write_file(&staging.join("provenance-review.md"), provenance)?;
    write_file(&staging.join("acquisition-report.json"), report)?;
    for (relative_path, bytes) in source_files {
        write_file(&staging.join("source").join(relative_path), bytes)?;
    }
    if output.exists() {
        fs::remove_dir(output)
            .map_err(|error| format!("remove confirmed-empty output directory: {error}"))?;
    }
    fs::rename(&staging, output)
        .map_err(|error| format!("publish listener-block output: {error}"))?;
    std::mem::forget(guard);
    Ok(())
}

fn write_file(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("create {}: {error}", parent.display()))?;
    }
    fs::write(path, bytes).map_err(|error| format!("write {}: {error}", path.display()))
}

struct StagingGuard(PathBuf);

impl Drop for StagingGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[derive(Deserialize)]
struct TransferCalibrationReport {
    schema: String,
    status: String,
    decision: String,
    revision: String,
    selected_profile_id: String,
    holdout_gate: TransferHoldoutGate,
    spatial_participation: TransferSpatialResult,
}

#[derive(Deserialize)]
struct TransferHoldoutGate {
    passed: bool,
}

#[derive(Deserialize)]
struct TransferSpatialResult {
    decision: String,
    observed_rows_per_object: usize,
    required_rows_per_object: usize,
}

#[derive(Serialize)]
struct FrozenBlockProfile {
    profile: &'static str,
    dataset_object_id: &'static str,
    source_row_range: [usize; 2],
    row_count: usize,
    impact_vertex_id: usize,
    azimuth_degrees: u32,
    gantry_distance_offset_millimetres: u32,
    microphone_ids: Vec<usize>,
    listener_z_metres: [f64; BLOCK_ROW_COUNT],
    compressed_prefix_bytes: u64,
    compressed_prefix_sha256: &'static str,
    block_payload_sha256: &'static str,
    row_sha256: [&'static str; BLOCK_ROW_COUNT],
    allowed_claims: [&'static str; ALLOWED_CLAIMS.len()],
    prohibited_claims: [&'static str; PROHIBITED_CLAIMS.len()],
}

struct RowIdentity {
    row_index: usize,
    microphone_id: usize,
    listener_position_metres: [f64; 3],
}

#[derive(Serialize)]
struct ListenerRow {
    row_index: usize,
    microphone_id: usize,
    listener_condition_id: String,
    listener_position_metres: [f64; 3],
    payload_offset_bytes: usize,
    sample_count: usize,
    raw_f32le_sha256: String,
    peak_abs: f64,
    rms: f64,
}

#[derive(Serialize)]
struct ListenerBlockManifest<'a> {
    schema: &'static str,
    status: &'static str,
    profile: &'static str,
    frozen_profile_sha256: &'a str,
    source_repository_revision: &'static str,
    dataset_object_id: &'static str,
    object_id: &'static str,
    geometry_revision: &'static str,
    impact_position_id: &'static str,
    impact_position_metres: [f64; 3],
    azimuth_degrees: u32,
    gantry_distance_offset_millimetres: u32,
    listener_x_metres: f64,
    listener_y_metres: f64,
    sample_rate_hz: u32,
    sample_count_per_row: usize,
    row_count: usize,
    block_payload: FileIdentity,
    rows: Vec<ListenerRow>,
    metadata_array_sha256: BTreeMap<&'static str, String>,
    prerequisites: Prerequisites,
    row_order_proof: [&'static str; 3],
    allowed_claims: &'static [&'static str],
    prohibited_claims: &'static [&'static str],
    unavailable_components: &'static [&'static str],
}

#[derive(Serialize)]
struct FileIdentity {
    path: &'static str,
    sha256: &'static str,
    byte_count: usize,
}

#[derive(Serialize)]
struct Prerequisites {
    corpus_plan_report_sha256: &'static str,
    transfer_calibration_report_sha256: &'static str,
    preprocess_measurements_sha256: &'static str,
    preprocess_annotations_sha256: &'static str,
}

#[derive(Serialize)]
struct ListenerBlockReport<'a> {
    schema: &'static str,
    status: &'static str,
    decision: &'static str,
    claim: &'static str,
    profile: &'static str,
    frozen_profile_sha256: &'a str,
    manifest_sha256: &'a str,
    provenance_sha256: &'a str,
    transfer_calibration_report_sha256: &'static str,
    archive_url: &'static str,
    archive_content_length: u64,
    central_directory_sha256: &'static str,
    compressed_prefix_bytes: u64,
    compressed_prefix_sha256: &'static str,
    http_range_payload_bytes: u64,
    full_archive_fraction: f64,
    row_count: usize,
    distinct_microphone_count: usize,
    same_impact_vertex: bool,
    same_azimuth: bool,
    same_distance_offset: bool,
    spatial_evaluation: &'static str,
    allowed_claims: &'static [&'static str],
    prohibited_claims: &'static [&'static str],
    next_action: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_profile_has_distinct_rows_and_only_acquisition_credit() {
        let profile = frozen_block_profile();
        assert_eq!(profile.row_count, 15);
        assert_eq!(profile.microphone_ids, (0..15).collect::<Vec<_>>());
        assert_eq!(
            profile
                .row_sha256
                .iter()
                .copied()
                .collect::<BTreeSet<_>>()
                .len(),
            15
        );
        assert!(PROHIBITED_CLAIMS.contains(&"spatial_model_validation"));
        assert!(!ALLOWED_CLAIMS.contains(&"spatial_model_validation"));
    }

    #[test]
    fn transfer_prerequisite_rejects_semantic_drift() {
        let report = serde_json::json!({
            "schema": TRANSFER_REPORT_SCHEMA,
            "status": "Validated",
            "decision": "RelativeModalDampingRejectedSpatialUnavailable",
            "revision": "v2",
            "selected_profile_id": "injective-modal-16-fft65536-v2",
            "holdout_gate": {"passed": true},
            "spatial_participation": {
                "decision": "NotEvaluableSingleListenerRowPerObject",
                "observed_rows_per_object": 1,
                "required_rows_per_object": 2
            }
        });
        let bytes = serde_json::to_vec(&report).expect("serializes");
        let parsed: TransferCalibrationReport =
            serde_json::from_slice(&bytes).expect("parses fixture");
        assert!(validate_transfer_report_semantics(&parsed).is_err());
    }
}
