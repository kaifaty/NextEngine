use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::super::{
    ArtifactReport, FileRef, MAX_REFERENCED_FILE_BYTES, canonical_external_file, read_bounded_file,
    resolve_artifact, validate_file_ref,
};
use super::{AudioReport, InventoryEntry, RecordingKind};

#[cfg(test)]
mod tests;

const REALIMPACT_ADAPTER_SCHEMA: &str = "realimpact_force_deconvolved_transfer_v1";
const REALIMPACT_REPOSITORY_REVISION: &str = "commit-fca2bd6cbb7e9f96ac61328d2a0d51594bf01987";
const REALIMPACT_ARCHIVE_PREFIX: &str = "https://downloads.cs.stanford.edu/viscam/RealImpact/";
const REALIMPACT_README_SHA256: &str =
    "3dd228b826651745f0cba8c8bfc0ff142f8fb4d9574f5adda91c272ffd8a2049";
const REALIMPACT_LICENSE_SHA256: &str =
    "328ed037c524ac2f73c859183ca6bf07d034dc28dae918fb3897051a6fd8b937";
const REALIMPACT_MEASUREMENTS_SHA256: &str =
    "db55f2017a037fb50a7b8b59542e85e35a7c7d5581b15fb75b25dbdf67737bbc";
const REALIMPACT_ANNOTATIONS_SHA256: &str =
    "66c4ab81d39a1ef4e26a72561dd2fa5238224f31ed089b27ca9a80d33852985a";
const REALIMPACT_DOWNLOAD_SHA256: &str =
    "4d6c2d7967d7dc2b8c56c7bfd7fe1550202099b6a7b1207aadc40e19d553a45a";
const REALIMPACT_DATASET_OBJECT_ID: &str = "94_GlassGoblet";
const REALIMPACT_ROW_INDEX: usize = 0;
const REALIMPACT_METADATA_SHA256: &str =
    "b9fd65f455df26c66da5bcd04d6ea0bbb580835c42932bf2167cf3d5f4861280";
const REALIMPACT_TRANSFER_SHA256: &str =
    "15c87b87423e71177e9e3b2ffd3fb0b2ea8ab7c5cbff071f519b2ddda3df325b";
const REALIMPACT_PROVENANCE_SHA256: &str =
    "d16716cafd8ad41d569415130dbdf82383f87d1025dac61c01ae2595f7f9cc51";
const REALIMPACT_GREEN_GOBLET_DATASET_OBJECT_ID: &str = "93_GreenGoblet";
const REALIMPACT_GREEN_GOBLET_METADATA_SHA256: &str =
    "a5c129535823c1422885a4adc3f27dcab36e8f959893d13b63b2f8efbd1bb649";
const REALIMPACT_GREEN_GOBLET_TRANSFER_SHA256: &str =
    "104dd97391bf6319ccbd4dfdf48569be58097bf1f90cf8cea3ae3cb2f7498ec9";
const REALIMPACT_GREEN_GOBLET_PROVENANCE_SHA256: &str =
    "25f01b1c8fb5b3cf6a9aa474393d9b00393dc705937c7b4148fedb7d1c7a4262";
const REALIMPACT_BLUE_BOWL_DATASET_OBJECT_ID: &str = "6_Bowl";
const REALIMPACT_BLUE_BOWL_METADATA_SHA256: &str =
    "f3dc97454a5888ec4a33488703d19e8fa6d5827a76095543c61d3235d3dc5c10";
const REALIMPACT_BLUE_BOWL_TRANSFER_SHA256: &str =
    "2640223087e3c719f6c2175eba35a652941a09f98209aa2acf93e43b37f1aa23";
const REALIMPACT_BLUE_BOWL_PROVENANCE_SHA256: &str =
    "1e1a7c931fbfb227bffe0df39e684e401adacedf3d2fcafd42f5b989662efcc4";
const REALIMPACT_SHELL_PLATE_DATASET_OBJECT_ID: &str = "51_ShellPlate";
const REALIMPACT_SHELL_PLATE_METADATA_SHA256: &str =
    "0847a18433bf3a3bb9586888cfd9ad9b90d4716c0294e501268ca00f23f69a15";
const REALIMPACT_SHELL_PLATE_TRANSFER_SHA256: &str =
    "e795d04f6bfe12414dd6499d2e29f2772f63ce1784f4d5f6ecdda681b0c31219";
const REALIMPACT_SHELL_PLATE_PROVENANCE_SHA256: &str =
    "66e572a1e69069907233172177662082bad7ebdb2bdc179be0efc32ef4c6cf25";
const REALIMPACT_CAPABILITIES: [&str; 6] = [
    "force_deconvolved_transfer",
    "geometry",
    "impact_position",
    "listener_position",
    "object_identity",
    "real_recording",
];
const REALIMPACT_UNAVAILABLE: [&str; 4] = [
    "force-profile-bytes",
    "material-composition-revision",
    "repeat-recording-identity",
    "support-fixture-revision",
];

#[derive(Clone, Copy)]
struct FrozenRealImpactPilot {
    dataset_object_id: &'static str,
    row_index: usize,
    metadata_sha256: &'static str,
    transfer_sha256: &'static str,
    provenance_sha256: &'static str,
}

const REALIMPACT_PILOTS: [FrozenRealImpactPilot; 4] = [
    FrozenRealImpactPilot {
        dataset_object_id: REALIMPACT_DATASET_OBJECT_ID,
        row_index: REALIMPACT_ROW_INDEX,
        metadata_sha256: REALIMPACT_METADATA_SHA256,
        transfer_sha256: REALIMPACT_TRANSFER_SHA256,
        provenance_sha256: REALIMPACT_PROVENANCE_SHA256,
    },
    FrozenRealImpactPilot {
        dataset_object_id: REALIMPACT_GREEN_GOBLET_DATASET_OBJECT_ID,
        row_index: 0,
        metadata_sha256: REALIMPACT_GREEN_GOBLET_METADATA_SHA256,
        transfer_sha256: REALIMPACT_GREEN_GOBLET_TRANSFER_SHA256,
        provenance_sha256: REALIMPACT_GREEN_GOBLET_PROVENANCE_SHA256,
    },
    FrozenRealImpactPilot {
        dataset_object_id: REALIMPACT_BLUE_BOWL_DATASET_OBJECT_ID,
        row_index: 0,
        metadata_sha256: REALIMPACT_BLUE_BOWL_METADATA_SHA256,
        transfer_sha256: REALIMPACT_BLUE_BOWL_TRANSFER_SHA256,
        provenance_sha256: REALIMPACT_BLUE_BOWL_PROVENANCE_SHA256,
    },
    FrozenRealImpactPilot {
        dataset_object_id: REALIMPACT_SHELL_PLATE_DATASET_OBJECT_ID,
        row_index: 0,
        metadata_sha256: REALIMPACT_SHELL_PLATE_METADATA_SHA256,
        transfer_sha256: REALIMPACT_SHELL_PLATE_TRANSFER_SHA256,
        provenance_sha256: REALIMPACT_SHELL_PLATE_PROVENANCE_SHA256,
    },
];

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "schema", deny_unknown_fields)]
pub(super) enum SourceAdapterProfile {
    #[serde(rename = "realimpact_force_deconvolved_transfer_v1")]
    RealImpactForceDeconvolvedTransferV1 {
        repository_revision: String,
        dataset_object_id: String,
        row_index: usize,
        repository_readme: FileRef,
        repository_license: FileRef,
        preprocess_measurements: FileRef,
        preprocess_annotations: FileRef,
        dataset_download_script: FileRef,
    },
}

#[derive(Debug, Serialize)]
pub(super) struct AdapterEvidenceReport {
    schema: &'static str,
    evidence_tier: &'static str,
    validated_capabilities: &'static [&'static str],
    repository_revision: String,
    source_artifacts: Vec<SourceArtifactEvidence>,
    archive_url: String,
    archive_content_length: u64,
    archive_etag: String,
    archive_last_modified: String,
    archive_central_directory_sha256: String,
    dataset_object_id: String,
    material_label: String,
    mesh_sha256: String,
    mesh_vertex_count: usize,
    row_index: usize,
    transfer_array_shape: [usize; 2],
    impact_vertex_id: usize,
    impact_position_metres: [f64; 3],
    listener_position_metres: [f64; 3],
    sample_rate_hz: u32,
    sample_count: usize,
    transfer_sha256: String,
    unavailable_components: &'static [&'static str],
}

#[derive(Debug, Serialize)]
struct SourceArtifactEvidence {
    role: &'static str,
    sha256: String,
    byte_count: usize,
}

pub(super) fn validate_declaration(
    entry: &InventoryEntry,
    require_typed_e2_adapter: bool,
) -> Result<(), String> {
    match (&entry.source_adapter, require_typed_e2_adapter) {
        (None, false) => Ok(()),
        (Some(_), false) => Err("source_adapter requires corpus-inventory manifest v2".to_owned()),
        (None, true)
            if matches!(
                entry.recording_kind,
                RecordingKind::ControlledRealForceDeconvolvedTransfer
            ) =>
        {
            Err(format!(
                "deconvolved inventory entry {} requires a typed source adapter",
                entry.id
            ))
        }
        (None, true) => Ok(()),
        (
            Some(SourceAdapterProfile::RealImpactForceDeconvolvedTransferV1 {
                repository_revision,
                dataset_object_id,
                row_index,
                repository_readme,
                repository_license,
                preprocess_measurements,
                preprocess_annotations,
                dataset_download_script,
            }),
            true,
        ) => {
            if !matches!(
                entry.recording_kind,
                RecordingKind::ControlledRealForceDeconvolvedTransfer
            ) {
                return Err(format!(
                    "REALIMPACT adapter on entry {} requires a deconvolved transfer",
                    entry.id
                ));
            }
            if repository_revision != REALIMPACT_REPOSITORY_REVISION {
                return Err(format!(
                    "REALIMPACT adapter on entry {} requires repository revision {REALIMPACT_REPOSITORY_REVISION}",
                    entry.id
                ));
            }
            validate_dataset_object_id(dataset_object_id)?;
            let pilot = REALIMPACT_PILOTS
                .iter()
                .find(|pilot| {
                    dataset_object_id == pilot.dataset_object_id && *row_index == pilot.row_index
                })
                .ok_or_else(|| {
                    format!(
                        "REALIMPACT adapter v1 has no frozen pilot for {dataset_object_id} row {row_index} on entry {}",
                        entry.id
                    )
                })?;
            for (actual, expected, role) in [
                (
                    entry.acquisition_metadata.sha256.as_str(),
                    pilot.metadata_sha256,
                    "acquisition metadata",
                ),
                (
                    entry.audio_payload.sha256.as_str(),
                    pilot.transfer_sha256,
                    "transfer payload",
                ),
                (
                    entry.provenance_review.sha256.as_str(),
                    pilot.provenance_sha256,
                    "provenance review",
                ),
            ] {
                if actual != expected {
                    return Err(format!(
                        "REALIMPACT adapter v1 {role} is not the frozen {dataset_object_id} pilot on entry {}",
                        entry.id
                    ));
                }
            }
            for (reference, expected_sha256, role) in [
                (
                    repository_readme,
                    REALIMPACT_README_SHA256,
                    "REALIMPACT repository README",
                ),
                (
                    repository_license,
                    REALIMPACT_LICENSE_SHA256,
                    "REALIMPACT repository license",
                ),
                (
                    preprocess_measurements,
                    REALIMPACT_MEASUREMENTS_SHA256,
                    "REALIMPACT measurement preprocessing source",
                ),
                (
                    preprocess_annotations,
                    REALIMPACT_ANNOTATIONS_SHA256,
                    "REALIMPACT annotation preprocessing source",
                ),
                (
                    dataset_download_script,
                    REALIMPACT_DOWNLOAD_SHA256,
                    "REALIMPACT dataset download source",
                ),
            ] {
                validate_file_ref(reference, role)?;
                if reference.sha256 != expected_sha256 {
                    return Err(format!(
                        "{role} must match frozen REALIMPACT revision {REALIMPACT_REPOSITORY_REVISION}"
                    ));
                }
            }
            Ok(())
        }
    }
}

pub(super) fn audit(
    root: &Path,
    manifest_directory: &Path,
    entry: &InventoryEntry,
    audio: &AudioReport,
) -> Result<Option<AdapterEvidenceReport>, String> {
    let Some(SourceAdapterProfile::RealImpactForceDeconvolvedTransferV1 {
        repository_revision,
        dataset_object_id,
        row_index,
        repository_readme,
        repository_license,
        preprocess_measurements,
        preprocess_annotations,
        dataset_download_script,
    }) = &entry.source_adapter
    else {
        return Ok(None);
    };

    let (readme, readme_bytes) = read_source_artifact(
        root,
        manifest_directory,
        repository_readme,
        "REALIMPACT repository README",
    )?;
    let (license, license_bytes) = read_source_artifact(
        root,
        manifest_directory,
        repository_license,
        "REALIMPACT repository license",
    )?;
    let (measurements, measurement_bytes) = read_source_artifact(
        root,
        manifest_directory,
        preprocess_measurements,
        "REALIMPACT measurement preprocessing source",
    )?;
    let (annotations, annotation_bytes) = read_source_artifact(
        root,
        manifest_directory,
        preprocess_annotations,
        "REALIMPACT annotation preprocessing source",
    )?;
    let (download, download_bytes) = read_source_artifact(
        root,
        manifest_directory,
        dataset_download_script,
        "REALIMPACT dataset download source",
    )?;
    validate_source_text(
        &readme_bytes,
        &measurement_bytes,
        &annotation_bytes,
        &download_bytes,
    )?;
    if !license_bytes.starts_with(b"MIT License") {
        return Err("REALIMPACT repository license is not the frozen MIT text".to_owned());
    }

    let metadata_path = canonical_external_file(
        root,
        &manifest_directory.join(&entry.acquisition_metadata.path),
        "REALIMPACT acquisition metadata",
    )?;
    let metadata_bytes = read_bounded_file(
        &metadata_path,
        MAX_REFERENCED_FILE_BYTES,
        "REALIMPACT acquisition metadata",
    )?;
    let metadata: RealImpactMetadata = serde_json::from_slice(&metadata_bytes)
        .map_err(|error| format!("parse {}: {error}", metadata_path.display()))?;
    validate_metadata(entry, audio, dataset_object_id, *row_index, &metadata)?;

    Ok(Some(AdapterEvidenceReport {
        schema: REALIMPACT_ADAPTER_SCHEMA,
        evidence_tier: "E2TransferResponse",
        validated_capabilities: &REALIMPACT_CAPABILITIES,
        repository_revision: repository_revision.clone(),
        source_artifacts: vec![
            source_evidence("dataset_download_script", download),
            source_evidence("preprocess_annotations", annotations),
            source_evidence("preprocess_measurements", measurements),
            source_evidence("repository_license", license),
            source_evidence("repository_readme", readme),
        ],
        archive_url: metadata.archive.url,
        archive_content_length: metadata.archive.content_length,
        archive_etag: metadata.archive.etag,
        archive_last_modified: metadata.archive.last_modified,
        archive_central_directory_sha256: metadata.archive.central_directory_sha256,
        dataset_object_id: metadata.object.dataset_object_id,
        material_label: metadata.object.material_label,
        mesh_sha256: metadata.object.mesh_sha256,
        mesh_vertex_count: metadata.object.mesh_vertex_count,
        row_index: metadata.pilot_row.row_index,
        transfer_array_shape: metadata.audio_entry.shape,
        impact_vertex_id: metadata.pilot_row.impact_vertex_id,
        impact_position_metres: metadata.pilot_row.impact_position_metres,
        listener_position_metres: metadata.pilot_row.listener_position_metres,
        sample_rate_hz: metadata.acquisition.sample_rate_hz,
        sample_count: metadata.pilot_row.sample_count,
        transfer_sha256: metadata.pilot_row.raw_f32le_sha256,
        unavailable_components: &REALIMPACT_UNAVAILABLE,
    }))
}

fn source_evidence(role: &'static str, artifact: ArtifactReport) -> SourceArtifactEvidence {
    SourceArtifactEvidence {
        role,
        sha256: artifact.sha256,
        byte_count: artifact.byte_count,
    }
}

fn read_source_artifact(
    root: &Path,
    manifest_directory: &Path,
    reference: &FileRef,
    role: &str,
) -> Result<(ArtifactReport, Vec<u8>), String> {
    let artifact = resolve_artifact(root, manifest_directory, reference, role)?;
    let path = canonical_external_file(root, &manifest_directory.join(&reference.path), role)?;
    let bytes = fs::read(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
    Ok((artifact, bytes))
}

fn validate_source_text(
    readme: &[u8],
    measurements: &[u8],
    annotations: &[u8],
    download: &[u8],
) -> Result<(), String> {
    let checks = [
        (
            readme,
            b"We are currently working on packaging and releasing the remainder of our code as well as our raw dataset".as_slice(),
            "repository raw-data boundary",
        ),
        (
            measurements,
            b"hammers_newtons = 1.11 * (1/.02558) * hammers_norm".as_slice(),
            "force calibration",
        ),
        (
            measurements,
            b"deconvolved = deconvolve_hammer(audios_norm, hammers_norm)".as_slice(),
            "force deconvolution",
        ),
        (
            measurements,
            b"preprocessed/deconvolved_0db.npy".as_slice(),
            "deconvolved transfer output",
        ),
        (
            measurements,
            b"preprocessed/listenerXYZ.npy".as_slice(),
            "listener coordinate output",
        ),
        (
            measurements,
            b"preprocessed/vertexXYZ.npy".as_slice(),
            "impact coordinate output",
        ),
        (
            annotations,
            b"preprocessed/angle.npy".as_slice(),
            "angle annotation output",
        ),
        (
            annotations,
            b"preprocessed/distance.npy".as_slice(),
            "distance annotation output",
        ),
        (
            annotations,
            b"preprocessed/micID.npy".as_slice(),
            "microphone annotation output",
        ),
        (
            download,
            b"downloads.cs.stanford.edu/viscam/RealImpact/$line.zip".as_slice(),
            "official archive download path",
        ),
    ];
    for (haystack, needle, role) in checks {
        if !haystack
            .windows(needle.len())
            .any(|window| window == needle)
        {
            return Err(format!("REALIMPACT source is missing {role}"));
        }
    }
    Ok(())
}

fn validate_metadata(
    entry: &InventoryEntry,
    audio: &AudioReport,
    dataset_object_id: &str,
    row_index: usize,
    metadata: &RealImpactMetadata,
) -> Result<(), String> {
    if metadata.schema != "nextengine.experimental-realimpact-bounded-import.metadata.v1"
        || metadata.status != "development_pilot_partial_source"
        || !is_date_prefix(&metadata.retrieved_at)
    {
        return Err("REALIMPACT acquisition metadata has an unsupported identity".to_owned());
    }
    if metadata.object.dataset_object_id != dataset_object_id
        || metadata.pilot_row.row_index != row_index
    {
        return Err(format!(
            "REALIMPACT adapter identity does not match entry {} metadata",
            entry.id
        ));
    }
    let expected_archive_url = format!("{REALIMPACT_ARCHIVE_PREFIX}{dataset_object_id}.zip");
    if metadata.archive.url != expected_archive_url
        || metadata.archive.content_length == 0
        || metadata.archive.etag.is_empty()
        || metadata.archive.last_modified.len() < 10
        || !metadata.archive.last_modified.is_ascii()
        || !is_date_prefix(&metadata.archive.last_modified)
    {
        return Err(format!(
            "REALIMPACT archive identity is invalid for entry {}",
            entry.id
        ));
    }
    validate_sha256(
        &metadata.archive.central_directory_sha256,
        "REALIMPACT central directory",
    )?;
    validate_sha256(&metadata.object.mesh_sha256, "REALIMPACT mesh")?;
    if metadata.object.material_label != entry.material_family
        || metadata.object.mesh_entry != format!("{dataset_object_id}/preprocessed/transformed.obj")
        || metadata.object.mesh_vertex_count == 0
    {
        return Err(format!(
            "REALIMPACT object metadata does not match entry {}",
            entry.id
        ));
    }
    validate_bounds(
        metadata.object.mesh_bbox_min_metres,
        "REALIMPACT mesh minimum",
    )?;
    validate_bounds(
        metadata.object.mesh_bbox_max_metres,
        "REALIMPACT mesh maximum",
    )?;
    if metadata
        .object
        .mesh_bbox_min_metres
        .into_iter()
        .zip(metadata.object.mesh_bbox_max_metres)
        .any(|(minimum, maximum)| minimum >= maximum)
    {
        return Err("REALIMPACT mesh bounds are empty".to_owned());
    }
    let expected_geometry_revision = format!("mesh-{}-v1", &metadata.object.mesh_sha256[..12]);
    let expected_object_id = format!(
        "realimpact-{}",
        dataset_object_id.replace('_', "-").to_ascii_lowercase()
    );
    let expected_source_id = format!(
        "realimpact-{}-archive-{}",
        &REALIMPACT_REPOSITORY_REVISION[7..19],
        &metadata.archive.last_modified[..10]
    );
    if entry.geometry_revision != expected_geometry_revision
        || entry.object_id != expected_object_id
        || entry.source_id != expected_source_id
        || entry.support_condition != "thread-mesh-unversioned-in-archive"
        || entry.excitation_method != "instrumented-hammer-force-deconvolved"
    {
        return Err(format!(
            "REALIMPACT entry {} has inconsistent source axes",
            entry.id
        ));
    }
    validate_acquisition(metadata)?;
    validate_downloaded_hashes(&metadata.downloaded_npy_hashes)?;
    validate_audio_entry(metadata)?;
    validate_pilot_row(entry, audio, metadata)?;
    let expected_unavailable = REALIMPACT_UNAVAILABLE.map(str::to_owned).to_vec();
    if metadata.unavailable_components != expected_unavailable
        || entry.unavailable_components != expected_unavailable
    {
        return Err(format!(
            "REALIMPACT entry {} does not preserve unavailable claim axes",
            entry.id
        ));
    }
    Ok(())
}

fn validate_acquisition(metadata: &RealImpactMetadata) -> Result<(), String> {
    let acquisition = &metadata.acquisition;
    let listener_count = acquisition
        .azimuth_degrees
        .len()
        .checked_mul(acquisition.gantry_distance_offsets_millimetres.len())
        .and_then(|count| count.checked_mul(acquisition.microphone_ids.len()))
        .ok_or_else(|| "REALIMPACT listener count overflow".to_owned())?;
    if acquisition.sample_rate_hz != 48_000
        || acquisition.support_condition
            != "thread-mesh-described-in-realimpact-paper-unversioned-in-archive"
        || acquisition.excitation_method != "instrumented-impact-hammer-force-deconvolved-transfer"
        || acquisition.impact_vertex_count == 0
        || acquisition.listener_position_count != listener_count
        || acquisition.gantry_distance_offsets_millimetres.len()
            != acquisition.paper_listener_distances_metres.len()
        || !strictly_increasing(&acquisition.azimuth_degrees)
        || !strictly_increasing(&acquisition.gantry_distance_offsets_millimetres)
        || !strictly_increasing(&acquisition.microphone_ids)
    {
        return Err("REALIMPACT acquisition axes are inconsistent".to_owned());
    }
    if acquisition
        .paper_listener_distances_metres
        .iter()
        .any(|value| !value.is_finite() || *value <= 0.0)
    {
        return Err("REALIMPACT listener distances are invalid".to_owned());
    }
    Ok(())
}

fn validate_audio_entry(metadata: &RealImpactMetadata) -> Result<(), String> {
    let audio = &metadata.audio_entry;
    let expected_path = format!(
        "{}/preprocessed/deconvolved_0db.npy",
        metadata.object.dataset_object_id
    );
    let payload_bytes = audio.shape[0]
        .checked_mul(audio.shape[1])
        .and_then(|samples| samples.checked_mul(4))
        .ok_or_else(|| "REALIMPACT transfer array byte count overflow".to_owned())?;
    if audio.path != expected_path
        || audio.dtype != "f32_le"
        || audio.shape[0]
            != metadata
                .acquisition
                .impact_vertex_count
                .checked_mul(metadata.acquisition.listener_position_count)
                .ok_or_else(|| "REALIMPACT transfer row count overflow".to_owned())?
        || audio.compressed_bytes == 0
        || audio.uncompressed_bytes < payload_bytes as u64
        || audio.uncompressed_bytes - payload_bytes as u64 > 4_096
        || !is_lower_hex(&audio.zip_crc32, 8)
    {
        return Err("REALIMPACT transfer array metadata is inconsistent".to_owned());
    }
    Ok(())
}

fn validate_pilot_row(
    entry: &InventoryEntry,
    audio: &AudioReport,
    metadata: &RealImpactMetadata,
) -> Result<(), String> {
    let row = &metadata.pilot_row;
    let expected_listener_id = format!(
        "angle-{:03}-distance-{:04}mm-mic-{:02}",
        row.azimuth_degrees,
        230_u32
            .checked_add(row.gantry_distance_offset_millimetres)
            .ok_or_else(|| "REALIMPACT listener distance overflow".to_owned())?,
        row.microphone_id
    );
    if row.row_index >= metadata.audio_entry.shape[0]
        || row.sample_count != metadata.audio_entry.shape[1]
        || entry.sample_rate_hz != metadata.acquisition.sample_rate_hz
        || entry.sample_count != row.sample_count
        || entry.impact_position_id != format!("mesh-vertex-{}", row.impact_vertex_id)
        || entry.impact_position_metres != row.impact_position_metres
        || entry.listener_condition_id != expected_listener_id
        || entry.listener_position_metres != row.listener_position_metres
        || !metadata
            .acquisition
            .azimuth_degrees
            .contains(&row.azimuth_degrees)
        || !metadata
            .acquisition
            .gantry_distance_offsets_millimetres
            .contains(&row.gantry_distance_offset_millimetres)
        || !metadata
            .acquisition
            .microphone_ids
            .contains(&row.microphone_id)
    {
        return Err(format!(
            "REALIMPACT pilot row does not match entry {}",
            entry.id
        ));
    }
    validate_sha256(&row.raw_f32le_sha256, "REALIMPACT transfer row")?;
    validate_sha256(
        &row.normalized_audition_wav_sha256,
        "REALIMPACT audition WAV",
    )?;
    let expected_duration =
        row.sample_count as f64 / f64::from(metadata.acquisition.sample_rate_hz);
    if row.raw_f32le_sha256 != entry.audio_payload.sha256
        || row.raw_f32le_sha256 != audio.sha256
        || !approximately_equal(row.duration_seconds, expected_duration)
        || !approximately_equal(row.peak_abs, audio.peak_abs)
        || !approximately_equal(row.rms, audio.rms)
    {
        return Err(format!(
            "REALIMPACT transfer payload does not match entry {}",
            entry.id
        ));
    }
    Ok(())
}

fn validate_downloaded_hashes(hashes: &DownloadedNpyHashes) -> Result<(), String> {
    for (value, role) in [
        (&hashes.angle_npy, "angle.npy"),
        (&hashes.distance_npy, "distance.npy"),
        (&hashes.listener_xyz_npy, "listenerXYZ.npy"),
        (&hashes.mic_id_npy, "micID.npy"),
        (&hashes.vertex_id_npy, "vertexID.npy"),
        (&hashes.vertex_xyz_npy, "vertexXYZ.npy"),
    ] {
        validate_sha256(value, role)?;
    }
    Ok(())
}

fn validate_dataset_object_id(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 128
        || !value.is_ascii()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err("REALIMPACT dataset object id is invalid".to_owned());
    }
    Ok(())
}

fn validate_sha256(value: &str, role: &str) -> Result<(), String> {
    if !is_lower_hex(value, 64) {
        return Err(format!("{role} sha256 must be 64 lowercase hex digits"));
    }
    Ok(())
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn is_date_prefix(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 10
        && bytes[..10].iter().enumerate().all(|(index, byte)| {
            if index == 4 || index == 7 {
                *byte == b'-'
            } else {
                byte.is_ascii_digit()
            }
        })
}

fn strictly_increasing(values: &[u32]) -> bool {
    !values.is_empty() && values.windows(2).all(|pair| pair[0] < pair[1])
}

fn validate_bounds(values: [f64; 3], role: &str) -> Result<(), String> {
    if values
        .into_iter()
        .any(|value| !value.is_finite() || value.abs() > 1_000_000.0)
    {
        return Err(format!("{role} must contain finite bounded metres"));
    }
    Ok(())
}

fn approximately_equal(left: f64, right: f64) -> bool {
    left.is_finite()
        && right.is_finite()
        && (left - right).abs() <= 1.0e-12 * left.abs().max(right.abs()).max(1.0)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RealImpactMetadata {
    schema: String,
    status: String,
    retrieved_at: String,
    archive: ArchiveMetadata,
    object: ObjectMetadata,
    acquisition: AcquisitionMetadata,
    downloaded_npy_hashes: DownloadedNpyHashes,
    audio_entry: AudioEntryMetadata,
    pilot_row: PilotRowMetadata,
    unavailable_components: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ArchiveMetadata {
    url: String,
    content_length: u64,
    etag: String,
    last_modified: String,
    central_directory_sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ObjectMetadata {
    dataset_object_id: String,
    material_label: String,
    mesh_entry: String,
    mesh_sha256: String,
    mesh_vertex_count: usize,
    mesh_bbox_min_metres: [f64; 3],
    mesh_bbox_max_metres: [f64; 3],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AcquisitionMetadata {
    sample_rate_hz: u32,
    support_condition: String,
    excitation_method: String,
    impact_vertex_count: usize,
    listener_position_count: usize,
    azimuth_degrees: Vec<u32>,
    gantry_distance_offsets_millimetres: Vec<u32>,
    paper_listener_distances_metres: Vec<f64>,
    microphone_ids: Vec<u32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DownloadedNpyHashes {
    #[serde(rename = "angle.npy")]
    angle_npy: String,
    #[serde(rename = "distance.npy")]
    distance_npy: String,
    #[serde(rename = "listenerXYZ.npy")]
    listener_xyz_npy: String,
    #[serde(rename = "micID.npy")]
    mic_id_npy: String,
    #[serde(rename = "vertexID.npy")]
    vertex_id_npy: String,
    #[serde(rename = "vertexXYZ.npy")]
    vertex_xyz_npy: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AudioEntryMetadata {
    path: String,
    zip_crc32: String,
    compressed_bytes: u64,
    uncompressed_bytes: u64,
    dtype: String,
    shape: [usize; 2],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PilotRowMetadata {
    row_index: usize,
    impact_vertex_id: usize,
    impact_position_metres: [f64; 3],
    azimuth_degrees: u32,
    gantry_distance_offset_millimetres: u32,
    microphone_id: u32,
    listener_position_metres: [f64; 3],
    sample_count: usize,
    duration_seconds: f64,
    raw_f32le_sha256: String,
    peak_abs: f64,
    rms: f64,
    normalized_audition_wav_sha256: String,
}
