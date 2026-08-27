use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;

use super::*;

static NEXT_STAGING: AtomicU64 = AtomicU64::new(0);

#[derive(Serialize)]
pub(super) struct AcquisitionMetadata<'a> {
    schema: &'static str,
    status: &'static str,
    retrieved_at: &'static str,
    archive: ArchiveMetadata,
    object: ObjectMetadata<'a>,
    acquisition: AcquisitionMetadataFields<'a>,
    downloaded_npy_hashes: &'a BTreeMap<&'static str, String>,
    audio_entry: AudioEntryMetadata,
    pilot_row: PilotRowMetadata<'a>,
    unavailable_components: &'static [&'static str],
}

#[derive(Serialize)]
struct ArchiveMetadata {
    url: &'static str,
    content_length: u64,
    etag: &'static str,
    last_modified: &'static str,
    central_directory_sha256: &'static str,
}

#[derive(Serialize)]
struct ObjectMetadata<'a> {
    dataset_object_id: &'static str,
    material_label: &'static str,
    mesh_entry: &'static str,
    mesh_sha256: &'a str,
    mesh_vertex_count: usize,
    mesh_bbox_min_metres: [f64; 3],
    mesh_bbox_max_metres: [f64; 3],
}

#[derive(Serialize)]
pub(super) struct AcquisitionMetadataFields<'a> {
    sample_rate_hz: u32,
    support_condition: &'static str,
    excitation_method: &'static str,
    impact_vertex_count: usize,
    listener_position_count: usize,
    azimuth_degrees: &'a [u32],
    gantry_distance_offsets_millimetres: &'a [u32],
    paper_listener_distances_metres: [f64; 4],
    microphone_ids: &'a [u32],
}

#[derive(Serialize)]
struct AudioEntryMetadata {
    path: &'static str,
    zip_crc32: &'static str,
    compressed_bytes: u64,
    uncompressed_bytes: u64,
    dtype: &'static str,
    shape: [usize; 2],
}

#[derive(Serialize)]
struct PilotRowMetadata<'a> {
    row_index: usize,
    impact_vertex_id: usize,
    impact_position_metres: [f64; 3],
    azimuth_degrees: u32,
    gantry_distance_offset_millimetres: u32,
    microphone_id: u32,
    listener_position_metres: [f64; 3],
    sample_count: usize,
    duration_seconds: f64,
    raw_f32le_sha256: &'a str,
    peak_abs: f64,
    rms: f64,
    normalized_audition_wav_sha256: &'a str,
}

pub(super) fn acquisition_metadata<'a>(
    derived: &'a DerivedMetadata,
    row: &'a AudioRow,
    wav_sha256: &'a str,
) -> AcquisitionMetadata<'a> {
    AcquisitionMetadata {
        schema: "nextengine.experimental-realimpact-bounded-import.metadata.v1",
        status: "development_pilot_partial_source",
        retrieved_at: "2026-08-27",
        archive: ArchiveMetadata {
            url: ARCHIVE_URL,
            content_length: ARCHIVE_BYTES,
            etag: ARCHIVE_ETAG,
            last_modified: ARCHIVE_LAST_MODIFIED_ISO,
            central_directory_sha256: CENTRAL_SHA256,
        },
        object: ObjectMetadata {
            dataset_object_id: "93_GreenGoblet",
            material_label: "glass",
            mesh_entry: "93_GreenGoblet/preprocessed/transformed.obj",
            mesh_sha256: &derived.mesh_sha256,
            mesh_vertex_count: derived.mesh_vertex_count,
            mesh_bbox_min_metres: derived.mesh_bbox_min_metres,
            mesh_bbox_max_metres: derived.mesh_bbox_max_metres,
        },
        acquisition: AcquisitionMetadataFields {
            sample_rate_hz: 48_000,
            support_condition: "thread-mesh-described-in-realimpact-paper-unversioned-in-archive",
            excitation_method: "instrumented-impact-hammer-force-deconvolved-transfer",
            impact_vertex_count: derived.impact_vertex_count,
            listener_position_count: derived.listener_position_count,
            azimuth_degrees: &derived.azimuth_degrees,
            gantry_distance_offsets_millimetres: &derived.distance_offsets,
            paper_listener_distances_metres: [0.23, 0.56, 0.90, 1.23],
            microphone_ids: &derived.microphone_ids,
        },
        downloaded_npy_hashes: &derived.downloaded_hashes,
        audio_entry: AudioEntryMetadata {
            path: "93_GreenGoblet/preprocessed/deconvolved_0db.npy",
            zip_crc32: "d41ca14a",
            compressed_bytes: 2_308_350_969,
            uncompressed_bytes: 2_499_876_128,
            dtype: "f32_le",
            shape: [3_000, 208_323],
        },
        pilot_row: PilotRowMetadata {
            row_index: 0,
            impact_vertex_id: derived.impact_vertex_id,
            impact_position_metres: derived.impact_position,
            azimuth_degrees: 0,
            gantry_distance_offset_millimetres: 0,
            microphone_id: 0,
            listener_position_metres: derived.listener_position,
            sample_count: row.sample_count,
            duration_seconds: row.sample_count as f64 / 48_000.0,
            raw_f32le_sha256: &row.sha256,
            peak_abs: row.peak_abs,
            rms: row.rms,
            normalized_audition_wav_sha256: wav_sha256,
        },
        unavailable_components: &UNAVAILABLE_COMPONENTS,
    }
}

#[derive(Serialize)]
pub(super) struct InventoryManifest<'a> {
    schema: &'static str,
    inventory_id: &'static str,
    revision: &'static str,
    source_class: &'static str,
    scope: &'static str,
    corpus_plan_report: ManifestFileRef<'a>,
    entries: [InventoryEntry<'a>; 1],
}

#[derive(Serialize)]
struct ManifestFileRef<'a> {
    path: &'static str,
    sha256: &'a str,
}

#[derive(Serialize)]
struct InventoryEntry<'a> {
    id: &'static str,
    partition: &'static str,
    recording_kind: &'static str,
    domain_id: &'static str,
    material_family: &'static str,
    object_family_id: &'static str,
    object_id: &'static str,
    source_id: &'static str,
    geometry_revision: &'static str,
    support_condition: &'static str,
    excitation_method: &'static str,
    impact_position_id: &'static str,
    impact_position_metres: [f64; 3],
    listener_condition_id: &'static str,
    listener_position_metres: [f64; 3],
    sample_rate_hz: u32,
    sample_count: usize,
    audio_format: &'static str,
    audio_payload: ManifestFileRef<'a>,
    acquisition_metadata: ManifestFileRef<'a>,
    provenance_review: ManifestFileRef<'a>,
    unavailable_components: &'static [&'static str],
    source_adapter: SourceAdapter,
}

#[derive(Serialize)]
struct SourceAdapter {
    schema: &'static str,
    repository_revision: &'static str,
    dataset_object_id: &'static str,
    row_index: usize,
    repository_readme: StaticFileRef,
    repository_license: StaticFileRef,
    preprocess_measurements: StaticFileRef,
    preprocess_annotations: StaticFileRef,
    dataset_download_script: StaticFileRef,
}

#[derive(Serialize)]
struct StaticFileRef {
    path: &'static str,
    sha256: &'static str,
}

pub(super) fn inventory_manifest<'a>(
    derived: &DerivedMetadata,
    row: &'a AudioRow,
    metadata_sha256: &'a str,
    provenance_sha256: &'a str,
) -> InventoryManifest<'a> {
    InventoryManifest {
        schema: "nextengine.experimental-physical-sound-corpus-inventory.manifest.v2",
        inventory_id: "ps2-realimpact-greengoblet-e2-range-v1",
        revision: "v1",
        source_class: "rigid_impact",
        scope: "development_pilot",
        corpus_plan_report: ManifestFileRef {
            path: "corpus-plan-report.json",
            sha256: CORPUS_PLAN_SHA256,
        },
        entries: [InventoryEntry {
            id: "realimpact-greengoblet-row0000",
            partition: "dev",
            recording_kind: "controlled_real_force_deconvolved_transfer",
            domain_id: "realimpact-green-goblet-thread-mesh-transfer",
            material_family: "glass",
            object_family_id: "realimpact-green-goblet",
            object_id: "realimpact-93-greengoblet",
            source_id: "realimpact-fca2bd6cbb7e-archive-2023-04-10",
            geometry_revision: "mesh-96252fe02006-v1",
            support_condition: "thread-mesh-unversioned-in-archive",
            excitation_method: "instrumented-hammer-force-deconvolved",
            impact_position_id: "mesh-vertex-31676",
            impact_position_metres: derived.impact_position,
            listener_condition_id: "angle-000-distance-0230mm-mic-00",
            listener_position_metres: derived.listener_position,
            sample_rate_hz: 48_000,
            sample_count: row.sample_count,
            audio_format: "f32_le_mono",
            audio_payload: ManifestFileRef {
                path: "greengoblet-row0000-deconvolved.f32le",
                sha256: &row.sha256,
            },
            acquisition_metadata: ManifestFileRef {
                path: "acquisition-metadata.json",
                sha256: metadata_sha256,
            },
            provenance_review: ManifestFileRef {
                path: "provenance-review.md",
                sha256: provenance_sha256,
            },
            unavailable_components: &UNAVAILABLE_COMPONENTS,
            source_adapter: SourceAdapter {
                schema: "realimpact_force_deconvolved_transfer_v1",
                repository_revision: REPOSITORY_REVISION,
                dataset_object_id: "93_GreenGoblet",
                row_index: 0,
                repository_readme: StaticFileRef {
                    path: "source/README.md",
                    sha256: SOURCE_FILES[0].sha256,
                },
                repository_license: StaticFileRef {
                    path: "source/LICENSE",
                    sha256: SOURCE_FILES[1].sha256,
                },
                preprocess_measurements: StaticFileRef {
                    path: "source/preprocess_measurements.py",
                    sha256: SOURCE_FILES[2].sha256,
                },
                preprocess_annotations: StaticFileRef {
                    path: "source/preprocess_annotations.py",
                    sha256: SOURCE_FILES[3].sha256,
                },
                dataset_download_script: StaticFileRef {
                    path: "source/dataset/download.sh",
                    sha256: SOURCE_FILES[4].sha256,
                },
            },
        }],
    }
}

#[derive(Serialize)]
pub(super) struct AcquisitionReport<'a> {
    pub(super) schema: &'static str,
    pub(super) status: &'static str,
    pub(super) decision: &'static str,
    pub(super) profile: &'static str,
    pub(super) archive_url: &'static str,
    pub(super) archive_content_length: u64,
    pub(super) central_directory_sha256: &'static str,
    pub(super) http_range_payload_bytes: u64,
    pub(super) full_archive_fraction: f64,
    pub(super) dataset_object_id: &'static str,
    pub(super) row_index: usize,
    pub(super) row_sha256: &'a str,
    pub(super) metadata_sha256: &'a str,
    pub(super) provenance_sha256: &'a str,
    pub(super) audition_wav_sha256: &'a str,
    pub(super) corpus_manifest_sha256: String,
    pub(super) unavailable_components: &'static [&'static str],
}

pub(super) fn provenance_review() -> &'static str {
    "# REALIMPACT GreenGoblet bounded-range provenance review\n\n\
Status: development-only external E2 transfer pilot; corpus admission and\n\
redistribution are not authorized.\n\n\
Primary sources:\n\n\
- https://samuelpclarke.com/realimpact/\n\
- https://github.com/samuel-clarke/RealImpact\n\
- https://jiajunwu.com/papers/realimpact_cvpr.pdf\n\
- https://downloads.cs.stanford.edu/viscam/RealImpact/93_GreenGoblet.zip\n\n\
The official preprocessing code converts the synchronized hammer trace to\n\
newtons and deconvolves it from the 48 kHz microphone recordings. The published\n\
archive contains the object mesh, impact/listener coordinates and a 3000 x\n\
208323 float32 deconvolved transfer array. It does not contain the raw force\n\
profile, material-composition revision, repeat identity or a versioned support\n\
fixture, so the row remains fallback-only.\n\n\
This acquisition reads the 1295-byte ZIP central directory, six small NPY\n\
members, the compressed mesh and a fixed 1048576-byte prefix of the large raw-\n\
deflate transfer member. The prefix yields the NPY header and row 0 without\n\
downloading the 2311697935-byte archive. The row's impact coordinate matches\n\
mesh vertex 31676 exactly. Archive HTTP identity, central directory, entry\n\
metadata, decoded members, row payload and normalized audition WAV are all\n\
hash-closed.\n\n\
The repository's MIT file applies to the published repository code. This review\n\
does not infer matching redistribution permission for the recording archive;\n\
all extracted bytes stay outside Git and are used only for local research.\n"
}

pub(super) fn read_source_bundle(
    root: &Path,
    directory: &Path,
) -> Result<Vec<(&'static str, Vec<u8>)>, String> {
    SOURCE_FILES
        .iter()
        .map(|source| {
            let path = canonical_external_file(
                root,
                &directory.join(source.relative_path),
                "REALIMPACT frozen source artifact",
            )?;
            let bytes = read_bounded_file(&path, 1024 * 1024, "REALIMPACT source artifact")?;
            require_hash(&bytes, source.sha256, source.relative_path)?;
            Ok((source.relative_path, bytes))
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
pub(super) fn publish_output(
    output: &Path,
    source_files: &[(&str, Vec<u8>)],
    corpus_plan: &[u8],
    row: &[u8],
    wav: &[u8],
    metadata: &[u8],
    provenance: &[u8],
    manifest: &[u8],
    report: &[u8],
) -> Result<(), String> {
    let parent = output
        .parent()
        .ok_or_else(|| "output has no parent".to_owned())?;
    let sequence = NEXT_STAGING.fetch_add(1, Ordering::Relaxed);
    let staging = parent.join(format!(
        ".nextengine-realimpact-row-{}-{sequence}",
        std::process::id()
    ));
    fs::create_dir(&staging)
        .map_err(|error| format!("create REALIMPACT staging directory: {error}"))?;
    let guard = StagingGuard(staging.clone());
    write_file(&staging.join("corpus-plan-report.json"), corpus_plan)?;
    write_file(&staging.join("greengoblet-row0000-deconvolved.f32le"), row)?;
    write_file(&staging.join("greengoblet-row0000-audition.wav"), wav)?;
    write_file(&staging.join("acquisition-metadata.json"), metadata)?;
    write_file(&staging.join("provenance-review.md"), provenance)?;
    write_file(&staging.join("manifest.json"), manifest)?;
    write_file(&staging.join("acquisition-report.json"), report)?;
    for (relative_path, bytes) in source_files {
        write_file(&staging.join("source").join(relative_path), bytes)?;
    }
    if output.exists() {
        fs::remove_dir(output)
            .map_err(|error| format!("remove confirmed-empty output directory: {error}"))?;
    }
    fs::rename(&staging, output).map_err(|error| format!("publish REALIMPACT output: {error}"))?;
    std::mem::forget(guard);
    Ok(())
}

struct StagingGuard(PathBuf);

impl Drop for StagingGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn write_file(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("create {}: {error}", parent.display()))?;
    }
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|error| format!("create {}: {error}", path.display()))?;
    file.write_all(bytes)
        .map_err(|error| format!("write {}: {error}", path.display()))?;
    file.sync_all()
        .map_err(|error| format!("sync {}: {error}", path.display()))
}

pub(super) fn pretty_json(value: &impl Serialize) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("serialize REALIMPACT evidence: {error}"))?;
    bytes.push(b'\n');
    Ok(bytes)
}
