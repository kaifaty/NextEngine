use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use flate2::read::DeflateDecoder;
use serde::Serialize;

use super::evidence::{pretty_json, read_source_bundle};
use super::profiles::{GREEN_GOBLET_PROFILE_ID, frozen_profile};
use super::*;

pub(super) const PROFILE_ID: &str = "green-goblet-dense-listener-block-v3";

const MANIFEST_SCHEMA: &str = "nextengine.experimental-realimpact-dense-listener-block.manifest.v1";
const REPORT_SCHEMA: &str =
    "nextengine.experimental-realimpact-dense-listener-block-acquisition.report.v1";
const BLOCK_ROW_COUNT: usize = 600;
const MICROPHONES_PER_COLUMN: usize = 15;
const COLUMN_COUNT: usize = 40;
const COMPRESSED_PREFIX_BYTES: u64 = 512 * 1024 * 1024;
const COMPRESSED_PREFIX_SHA256: &str =
    "f8a48f8c71d2979a054a9cf116afa4e76195aee79688d480fe81f84ba1a02794";
const BLOCK_PAYLOAD_SHA256: &str =
    "381e958c53bf73b2aa62bf8dae874cf0167945a916c62514a9d2ab81e3c5188e";
const BLOCK_FILE_NAME: &str = "green-goblet-fixed-impact-listener-block-0000-0599.f32le";
const ANGLES_DEGREES: [i64; 10] = [0, 20, 40, 60, 80, 100, 120, 140, 160, 180];
const DISTANCE_OFFSETS_MILLIMETRES: [i64; 4] = [0, 333, 666, 1_000];
const QUERY_ANGLES_DEGREES: [i64; 3] = [40, 100, 160];
const LISTENER_Z_METRES: [f64; MICROPHONES_PER_COLUMN] = [
    -0.91, -0.78, -0.65, -0.52, -0.39, -0.26, -0.13, 0.0, 0.13, 0.26, 0.39, 0.52, 0.65, 0.78, 0.91,
];
const ALLOWED_CLAIMS: [&str; 4] = [
    "exact_row_to_published_listener_identity",
    "same_published_fixed_impact_vertex",
    "development_dense_listener_field_preflight",
    "relative_multi_listener_transfer_response",
];
const PROHIBITED_CLAIMS: [&str; 8] = [
    "absolute_amplitude_claim",
    "complex_field_quality_pass",
    "cross_object_spatial_generalization",
    "model_training_authorized",
    "physical_sound_pass",
    "production_corpus_admission",
    "runtime_content_role",
    "shadow_or_holdout_access",
];

static NEXT_STAGING: AtomicU64 = AtomicU64::new(0);

pub(super) fn run(root: &Path, request: &Request) -> Result<(), String> {
    let output = resolve_output_path(root, &request.output)?;
    require_empty_output(&output)?;
    let transfer_path = request
        .transfer_calibration_report
        .as_ref()
        .ok_or_else(|| {
            "dense listener-block profile requires --transfer-calibration-report <external-json>"
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
    listener_block::validate_transfer_report(&transfer_report)?;

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
    let compressed_prefix = fetch_entry(
        profile,
        &resolve,
        audio_spec,
        COMPRESSED_PREFIX_BYTES,
        &mut fetched_bytes,
    )?;
    require_hash(
        &compressed_prefix,
        COMPRESSED_PREFIX_SHA256,
        "Green Goblet dense listener compressed prefix",
    )?;
    let (block_payload, row_stats) = extract_dense_block(profile, &compressed_prefix)?;
    let identities = validate_dense_identities(profile, &raw_entries)?;

    let columns = build_columns();
    let rows = identities
        .into_iter()
        .zip(row_stats)
        .map(|(identity, stats)| DenseRow {
            row_index: identity.row_index,
            column_id: identity.column_id,
            split_role: split_role(identity.angle_degrees),
            azimuth_degrees: identity.angle_degrees,
            gantry_distance_offset_millimetres: identity.distance_offset_millimetres,
            microphone_id: identity.microphone_id,
            listener_position_metres: identity.listener_position_metres,
            payload_offset_bytes: identity.row_index * profile.audio_sample_count * 4,
            sample_count: profile.audio_sample_count,
            raw_f32le_sha256: stats.sha256,
            peak_abs: stats.peak_abs,
            rms: stats.rms,
        })
        .collect::<Vec<_>>();
    let metadata_array_sha256 = raw_entries
        .iter()
        .filter(|(name, _)| name.ends_with(".npy"))
        .map(|(name, bytes)| {
            (
                name.rsplit('/').next().expect("entry has a filename"),
                sha256_hex(bytes),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let frozen_profile = FrozenDenseProfile {
        profile: PROFILE_ID,
        dataset_object_id: profile.dataset_object_id,
        source_row_range: [0, BLOCK_ROW_COUNT - 1],
        row_count: BLOCK_ROW_COUNT,
        column_count: COLUMN_COUNT,
        microphones_per_column: MICROPHONES_PER_COLUMN,
        impact_vertex_id: profile.expected_impact_vertex_id,
        angles_degrees: ANGLES_DEGREES,
        distance_offsets_millimetres: DISTANCE_OFFSETS_MILLIMETRES,
        query_angles_degrees: QUERY_ANGLES_DEGREES,
        compressed_prefix_bytes: COMPRESSED_PREFIX_BYTES,
        compressed_prefix_sha256: COMPRESSED_PREFIX_SHA256,
        block_payload_sha256: BLOCK_PAYLOAD_SHA256,
        allowed_claims: ALLOWED_CLAIMS,
        prohibited_claims: PROHIBITED_CLAIMS,
    };
    let frozen_profile_bytes = pretty_json(&frozen_profile)?;
    let frozen_profile_sha256 = sha256_hex(&frozen_profile_bytes);
    let manifest = DenseManifest {
        schema: MANIFEST_SCHEMA,
        status: "development_dense_fixed_impact_block",
        profile: PROFILE_ID,
        frozen_profile_sha256: &frozen_profile_sha256,
        source_repository_revision: REPOSITORY_REVISION,
        dataset_object_id: profile.dataset_object_id,
        object_id: profile.object_id,
        geometry_revision: profile.geometry_revision,
        impact_position_id: profile.impact_position_id,
        impact_position_metres: profile.expected_impact_position,
        sample_rate_hz: 48_000,
        sample_count_per_row: profile.audio_sample_count,
        row_count: BLOCK_ROW_COUNT,
        column_count: COLUMN_COUNT,
        context_row_count: rows
            .iter()
            .filter(|row| row.split_role == "context")
            .count(),
        query_row_count: rows.iter().filter(|row| row.split_role == "query").count(),
        block_payload: FileIdentity {
            path: BLOCK_FILE_NAME,
            sha256: BLOCK_PAYLOAD_SHA256,
            byte_count: block_payload.len(),
        },
        columns,
        rows,
        metadata_array_sha256,
        representation_preflight: representation_preflight(),
        prerequisites: Prerequisites {
            corpus_plan_report_sha256: CORPUS_PLAN_SHA256,
            transfer_calibration_report_sha256: sha256_hex(&transfer_report),
            preprocess_measurements_sha256: SOURCE_FILES[2].sha256,
            preprocess_annotations_sha256: SOURCE_FILES[3].sha256,
        },
        allowed_claims: &ALLOWED_CLAIMS,
        prohibited_claims: &PROHIBITED_CLAIMS,
        unavailable_components: &UNAVAILABLE_COMPONENTS,
    };
    let manifest_bytes = pretty_json(&manifest)?;
    let manifest_sha256 = sha256_hex(&manifest_bytes);
    let provenance = provenance_review();
    let provenance_sha256 = sha256_hex(provenance.as_bytes());
    let report = DenseReport {
        schema: REPORT_SCHEMA,
        status: "Validated",
        decision: "DenseListenerBlockAcquiredPreflightRequired",
        claim: "FULL_600_POSITION_FIXED_IMPACT_DEVELOPMENT_BLOCK_ONLY / NO_MODEL_TRAINING_QUALITY_ADMISSION_OR_RUNTIME_AUTHORITY",
        profile: PROFILE_ID,
        frozen_profile_sha256: &frozen_profile_sha256,
        manifest_sha256: &manifest_sha256,
        provenance_sha256: &provenance_sha256,
        archive_url: profile.archive_url,
        archive_content_length: profile.archive_bytes,
        central_directory_sha256: profile.central_sha256,
        compressed_prefix_bytes: COMPRESSED_PREFIX_BYTES,
        compressed_prefix_sha256: COMPRESSED_PREFIX_SHA256,
        http_range_payload_bytes: fetched_bytes,
        full_archive_fraction: fetched_bytes as f64 / profile.archive_bytes as f64,
        block_payload_sha256: BLOCK_PAYLOAD_SHA256,
        block_payload_bytes: block_payload.len(),
        row_count: BLOCK_ROW_COUNT,
        column_count: COLUMN_COUNT,
        context_row_count: manifest.context_row_count,
        query_row_count: manifest.query_row_count,
        method_holdout_or_shadow_bytes_read: 0,
        optimizer_steps: 0,
        training_authorized: false,
        next_action: "run the frozen complex-field data/representation preflight twice before any optimizer step",
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
    println!(
        "REALIMPACT dense listener block output: {}",
        output.display()
    );
    println!("block payload sha256: {BLOCK_PAYLOAD_SHA256}");
    println!("manifest sha256: {manifest_sha256}");
    println!("report decision: DenseListenerBlockAcquiredPreflightRequired");
    Ok(())
}

fn extract_dense_block(
    profile: &FrozenProfile,
    compressed: &[u8],
) -> Result<(Vec<u8>, Vec<RowStats>), String> {
    let mut decoder = DeflateDecoder::new(compressed);
    let mut header = [0_u8; 128];
    decoder
        .read_exact(&mut header)
        .map_err(|error| format!("decompress REALIMPACT transfer NPY header: {error}"))?;
    validate_npy_header_prefix(&header, "<f4", &[3_000, profile.audio_sample_count])?;
    let block_bytes = BLOCK_ROW_COUNT
        .checked_mul(profile.audio_sample_count)
        .and_then(|value| value.checked_mul(4))
        .ok_or_else(|| "dense listener block byte count overflow".to_owned())?;
    let mut block = vec![0_u8; block_bytes];
    decoder
        .read_exact(&mut block)
        .map_err(|error| format!("decompress first 600 REALIMPACT transfer rows: {error}"))?;
    require_hash(
        &block,
        BLOCK_PAYLOAD_SHA256,
        "Green Goblet dense listener block payload",
    )?;
    let row_bytes = profile.audio_sample_count * 4;
    let rows = block
        .chunks_exact(row_bytes)
        .enumerate()
        .map(|(row_index, bytes)| analyze_row(row_index, bytes, profile.audio_sample_count))
        .collect::<Result<Vec<_>, _>>()?;
    if rows.len() != BLOCK_ROW_COUNT {
        return Err("dense listener block row count changed".to_owned());
    }
    Ok((block, rows))
}

fn analyze_row(row_index: usize, bytes: &[u8], sample_count: usize) -> Result<RowStats, String> {
    let mut peak_abs = 0.0_f64;
    let mut sum_squared = 0.0_f64;
    for sample in bytes.chunks_exact(4) {
        let value = f64::from(f32::from_le_bytes(sample.try_into().expect("four bytes")));
        if !value.is_finite() {
            return Err(format!(
                "REALIMPACT dense listener row {row_index} contains a non-finite sample"
            ));
        }
        peak_abs = peak_abs.max(value.abs());
        sum_squared += value * value;
    }
    Ok(RowStats {
        sha256: sha256_hex(bytes),
        peak_abs,
        rms: (sum_squared / sample_count as f64).sqrt(),
    })
}

fn validate_dense_identities(
    profile: &FrozenProfile,
    raw_entries: &BTreeMap<&'static str, Vec<u8>>,
) -> Result<Vec<RowIdentity>, String> {
    let vertex_xyz = f64_array(raw(raw_entries, "vertexXYZ.npy")?, &[3_000, 3])?;
    let listener_xyz = f64_array(raw(raw_entries, "listenerXYZ.npy")?, &[3_000, 3])?;
    let vertex_ids = i64_array(raw(raw_entries, "vertexID.npy")?, &[3_000])?;
    let microphone_ids = i64_array(raw(raw_entries, "micID.npy")?, &[3_000])?;
    let distances = i64_array(raw(raw_entries, "distance.npy")?, &[3_000])?;
    let angles = i64_array(raw(raw_entries, "angle.npy")?, &[3_000])?;
    let mut listener_positions = BTreeSet::new();
    let mut identities = Vec::with_capacity(BLOCK_ROW_COUNT);
    for row_index in 0..BLOCK_ROW_COUNT {
        let column_index = row_index / MICROPHONES_PER_COLUMN;
        let microphone_id = row_index % MICROPHONES_PER_COLUMN;
        let expected_angle = ANGLES_DEGREES[column_index / DISTANCE_OFFSETS_MILLIMETRES.len()];
        let expected_distance =
            DISTANCE_OFFSETS_MILLIMETRES[column_index % DISTANCE_OFFSETS_MILLIMETRES.len()];
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
        if vertex_ids[row_index] != profile.expected_impact_vertex_id as i64
            || impact != profile.expected_impact_position
            || angles[row_index] != expected_angle
            || distances[row_index] != expected_distance
            || microphone_ids[row_index] != microphone_id as i64
            || (listener[2] - LISTENER_Z_METRES[microphone_id]).abs() > 1.0e-12
            || !listener.iter().all(|value| value.is_finite())
            || !listener_positions.insert(listener.map(f64::to_bits))
        {
            return Err(format!(
                "REALIMPACT Green Goblet dense listener identity changed at row {row_index}"
            ));
        }
        identities.push(RowIdentity {
            row_index,
            column_id: column_id(expected_angle, expected_distance),
            angle_degrees: expected_angle,
            distance_offset_millimetres: expected_distance,
            microphone_id,
            listener_position_metres: listener,
        });
    }
    if listener_positions.len() != BLOCK_ROW_COUNT {
        return Err("dense listener positions are not unique".to_owned());
    }
    Ok(identities)
}

fn build_columns() -> Vec<DenseColumn> {
    ANGLES_DEGREES
        .iter()
        .flat_map(|angle| {
            DISTANCE_OFFSETS_MILLIMETRES
                .iter()
                .map(move |distance| (*angle, *distance))
        })
        .enumerate()
        .map(|(column_index, (angle, distance))| DenseColumn {
            column_index,
            column_id: column_id(angle, distance),
            split_role: split_role(angle),
            azimuth_degrees: angle,
            gantry_distance_offset_millimetres: distance,
            row_range: [
                column_index * MICROPHONES_PER_COLUMN,
                (column_index + 1) * MICROPHONES_PER_COLUMN - 1,
            ],
        })
        .collect()
}

fn column_id(angle_degrees: i64, distance_millimetres: i64) -> String {
    format!("angle-{angle_degrees:03}-distance-{distance_millimetres:04}mm")
}

fn split_role(angle_degrees: i64) -> &'static str {
    if QUERY_ANGLES_DEGREES.contains(&angle_degrees) {
        "query"
    } else {
        "context"
    }
}

fn representation_preflight() -> RepresentationPreflight {
    RepresentationPreflight {
        id: "complex-stft-sqrt-periodic-hann-2048-hop512-v1",
        target: "complex_pressure_rfft",
        sample_dtype: "float32le",
        feature_dtype: "complex64le",
        fft_length: 2_048,
        window_length: 2_048,
        hop_length: 512,
        window: "sqrt_periodic_hann",
        centering: "zero_pad_half_window_each_side",
        inverse: "irfft_overlap_add_divide_window_square_then_crop",
        shared_normalization: "context_420_rows_global_peak_to_0.92_pcm16_full_scale_applied_unchanged_to_query",
        split_unit: "complete_15_microphone_gantry_column",
        query_group: "all_columns_at_azimuth_degrees_40_100_160",
        context_group: "all_columns_at_remaining_published_azimuths",
        classical_controls: [
            "nearest_context_azimuth_same_distance_and_microphone",
            "linear_bracketing_azimuth_same_distance_and_microphone",
            "log_magnitude_shortest_arc_phase_bracketing_complex_stft",
        ],
        inverse_float_nrmse_db_max: -140.0,
        inverse_max_absolute_error_max: 1.0e-6,
        inverse_pcm_requirement: "maximum_absolute_difference_lte_1_lsb",
        optimizer_authorized: false,
    }
}

fn provenance_review() -> String {
    "# REALIMPACT Green Goblet dense fixed-impact listener acquisition\n\n\
Status: development-only external R2B data/representation preflight; no model,\n\
quality, admission or runtime authority.\n\n\
Primary sources:\n\n\
- https://samuelpclarke.com/realimpact/\n\
- https://github.com/samuel-clarke/RealImpact\n\
- https://openaccess.thecvf.com/content/CVPR2023/papers/Clarke_RealImpact_A_Dataset_of_Impact_Sound_Fields_for_Real_Objects_CVPR_2023_paper.pdf\n\
- https://downloads.cs.stanford.edu/viscam/RealImpact/93_GreenGoblet.zip\n\n\
The frozen source scripts and hash-verified arrays establish the first 600 rows\n\
as one impact vertex, 40 angle/distance gantry columns and 15 microphones per\n\
column. Query roles hold complete 40, 100 and 160 degree planes; no microphone\n\
within those columns is available to representation fitting. The command reads\n\
one exact 512 MiB compressed prefix and emits only the first 600 deconvolved\n\
float32 rows. Dataset payloads and derived artifacts remain outside Git.\n"
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
        .ok_or_else(|| "dense listener-block output has no parent".to_owned())?;
    let sequence = NEXT_STAGING.fetch_add(1, Ordering::Relaxed);
    let staging = parent.join(format!(
        ".nextengine-realimpact-dense-listener-block-{}-{sequence}",
        std::process::id()
    ));
    fs::create_dir(&staging)
        .map_err(|error| format!("create dense listener-block staging directory: {error}"))?;
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
            .map_err(|error| format!("remove confirmed-empty dense output: {error}"))?;
    }
    fs::rename(&staging, output)
        .map_err(|error| format!("publish dense listener-block output: {error}"))?;
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

struct RowStats {
    sha256: String,
    peak_abs: f64,
    rms: f64,
}

struct RowIdentity {
    row_index: usize,
    column_id: String,
    angle_degrees: i64,
    distance_offset_millimetres: i64,
    microphone_id: usize,
    listener_position_metres: [f64; 3],
}

#[derive(Serialize)]
struct FrozenDenseProfile {
    profile: &'static str,
    dataset_object_id: &'static str,
    source_row_range: [usize; 2],
    row_count: usize,
    column_count: usize,
    microphones_per_column: usize,
    impact_vertex_id: usize,
    angles_degrees: [i64; 10],
    distance_offsets_millimetres: [i64; 4],
    query_angles_degrees: [i64; 3],
    compressed_prefix_bytes: u64,
    compressed_prefix_sha256: &'static str,
    block_payload_sha256: &'static str,
    allowed_claims: [&'static str; ALLOWED_CLAIMS.len()],
    prohibited_claims: [&'static str; PROHIBITED_CLAIMS.len()],
}

#[derive(Serialize)]
struct FileIdentity {
    path: &'static str,
    sha256: &'static str,
    byte_count: usize,
}

#[derive(Serialize)]
struct DenseColumn {
    column_index: usize,
    column_id: String,
    split_role: &'static str,
    azimuth_degrees: i64,
    gantry_distance_offset_millimetres: i64,
    row_range: [usize; 2],
}

#[derive(Serialize)]
struct DenseRow {
    row_index: usize,
    column_id: String,
    split_role: &'static str,
    azimuth_degrees: i64,
    gantry_distance_offset_millimetres: i64,
    microphone_id: usize,
    listener_position_metres: [f64; 3],
    payload_offset_bytes: usize,
    sample_count: usize,
    raw_f32le_sha256: String,
    peak_abs: f64,
    rms: f64,
}

#[derive(Serialize)]
struct RepresentationPreflight {
    id: &'static str,
    target: &'static str,
    sample_dtype: &'static str,
    feature_dtype: &'static str,
    fft_length: usize,
    window_length: usize,
    hop_length: usize,
    window: &'static str,
    centering: &'static str,
    inverse: &'static str,
    shared_normalization: &'static str,
    split_unit: &'static str,
    query_group: &'static str,
    context_group: &'static str,
    classical_controls: [&'static str; 3],
    inverse_float_nrmse_db_max: f64,
    inverse_max_absolute_error_max: f64,
    inverse_pcm_requirement: &'static str,
    optimizer_authorized: bool,
}

#[derive(Serialize)]
struct Prerequisites {
    corpus_plan_report_sha256: &'static str,
    transfer_calibration_report_sha256: String,
    preprocess_measurements_sha256: &'static str,
    preprocess_annotations_sha256: &'static str,
}

#[derive(Serialize)]
struct DenseManifest<'a> {
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
    sample_rate_hz: u32,
    sample_count_per_row: usize,
    row_count: usize,
    column_count: usize,
    context_row_count: usize,
    query_row_count: usize,
    block_payload: FileIdentity,
    columns: Vec<DenseColumn>,
    rows: Vec<DenseRow>,
    metadata_array_sha256: BTreeMap<&'static str, String>,
    representation_preflight: RepresentationPreflight,
    prerequisites: Prerequisites,
    allowed_claims: &'static [&'static str],
    prohibited_claims: &'static [&'static str],
    unavailable_components: &'static [&'static str],
}

#[derive(Serialize)]
struct DenseReport<'a> {
    schema: &'static str,
    status: &'static str,
    decision: &'static str,
    claim: &'static str,
    profile: &'static str,
    frozen_profile_sha256: &'a str,
    manifest_sha256: &'a str,
    provenance_sha256: &'a str,
    archive_url: &'static str,
    archive_content_length: u64,
    central_directory_sha256: &'static str,
    compressed_prefix_bytes: u64,
    compressed_prefix_sha256: &'static str,
    http_range_payload_bytes: u64,
    full_archive_fraction: f64,
    block_payload_sha256: &'static str,
    block_payload_bytes: usize,
    row_count: usize,
    column_count: usize,
    context_row_count: usize,
    query_row_count: usize,
    method_holdout_or_shadow_bytes_read: usize,
    optimizer_steps: usize,
    training_authorized: bool,
    next_action: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grouped_split_holds_complete_angle_planes() {
        let columns = build_columns();
        assert_eq!(columns.len(), 40);
        assert_eq!(
            columns
                .iter()
                .filter(|column| column.split_role == "query")
                .count(),
            12
        );
        assert_eq!(
            columns
                .iter()
                .filter(|column| column.split_role == "context")
                .count(),
            28
        );
        for angle in QUERY_ANGLES_DEGREES {
            assert!(
                columns
                    .iter()
                    .filter(|column| column.azimuth_degrees == angle)
                    .all(|column| column.split_role == "query")
            );
        }
    }

    #[test]
    fn dense_profile_does_not_authorize_training_or_pass() {
        let representation = representation_preflight();
        assert!(!representation.optimizer_authorized);
        assert_eq!(representation.inverse_float_nrmse_db_max, -140.0);
        assert_eq!(representation.inverse_max_absolute_error_max, 1.0e-6);
        assert_eq!(
            representation.shared_normalization,
            "context_420_rows_global_peak_to_0.92_pcm16_full_scale_applied_unchanged_to_query"
        );
        assert_eq!(
            representation.inverse_pcm_requirement,
            "maximum_absolute_difference_lte_1_lsb"
        );
        assert!(PROHIBITED_CLAIMS.contains(&"model_training_authorized"));
        assert!(PROHIBITED_CLAIMS.contains(&"physical_sound_pass"));
        assert!(!ALLOWED_CLAIMS.contains(&"physical_sound_pass"));
        assert_eq!(BLOCK_ROW_COUNT, COLUMN_COUNT * MICROPHONES_PER_COLUMN);
    }

    #[test]
    fn rotated_listener_z_uses_a_numeric_tolerance() {
        let published_rotated_z = -0.909_999_999_999_999_9_f64;
        assert!((published_rotated_z - LISTENER_Z_METRES[0]).abs() <= 1.0e-12);
        assert!((-0.909_f64 - LISTENER_Z_METRES[0]).abs() > 1.0e-12);
    }
}
