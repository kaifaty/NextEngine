use super::super::{AudioFormat, InventoryEntry, Partition, RecordingKind};
use super::*;

#[test]
fn v2_deconvolved_transfer_requires_frozen_typed_source_identity() {
    let mut entry = entry_fixture();
    assert!(
        validate_declaration(&entry, true)
            .expect_err("missing source adapter rejects")
            .contains("requires a typed source adapter")
    );

    entry.source_adapter = Some(adapter_fixture());
    validate_declaration(&entry, true).expect("frozen source adapter validates");

    let Some(SourceAdapterProfile::RealImpactForceDeconvolvedTransferV1 {
        repository_readme, ..
    }) = &mut entry.source_adapter
    else {
        panic!("REALIMPACT adapter fixture");
    };
    repository_readme.sha256 = "a".repeat(64);
    assert!(
        validate_declaration(&entry, true)
            .expect_err("unbound source revision rejects")
            .contains("must match frozen REALIMPACT revision")
    );
}

#[test]
fn realimpact_metadata_grants_only_bound_transfer_axes() {
    let entry = entry_fixture_with_adapter();
    let audio = AudioReport {
        format: "f32_le_mono",
        sha256: REALIMPACT_TRANSFER_SHA256.to_owned(),
        byte_count: 16,
        peak_abs: 0.5,
        rms: 0.279_508_497_187_473_7,
    };
    let mut metadata = metadata_fixture();
    validate_metadata(&entry, &audio, "94_GlassGoblet", 0, &metadata)
        .expect("consistent typed metadata validates");

    metadata.pilot_row.listener_position_metres[2] = -0.8;
    assert!(
        validate_metadata(&entry, &audio, "94_GlassGoblet", 0, &metadata)
            .expect_err("coordinate mismatch rejects")
            .contains("pilot row does not match")
    );
}

#[test]
fn declaration_accepts_the_independently_fetched_green_goblet_row() {
    let mut entry = entry_fixture();
    entry.id = "realimpact-greengoblet-row0000".to_owned();
    entry.object_id = "realimpact-93-greengoblet".to_owned();
    entry.audio_payload.sha256 = REALIMPACT_GREEN_GOBLET_TRANSFER_SHA256.to_owned();
    entry.acquisition_metadata.sha256 = REALIMPACT_GREEN_GOBLET_METADATA_SHA256.to_owned();
    entry.provenance_review.sha256 = REALIMPACT_GREEN_GOBLET_PROVENANCE_SHA256.to_owned();
    let mut adapter = adapter_fixture();
    let SourceAdapterProfile::RealImpactForceDeconvolvedTransferV1 {
        dataset_object_id, ..
    } = &mut adapter;
    *dataset_object_id = REALIMPACT_GREEN_GOBLET_DATASET_OBJECT_ID.to_owned();
    entry.source_adapter = Some(adapter);
    validate_declaration(&entry, true).expect("frozen GreenGoblet source adapter validates");

    let Some(SourceAdapterProfile::RealImpactForceDeconvolvedTransferV1 { row_index, .. }) =
        &mut entry.source_adapter
    else {
        panic!("REALIMPACT adapter fixture");
    };
    *row_index = 1;
    assert!(
        validate_declaration(&entry, true)
            .expect_err("unfrozen row rejects")
            .contains("has no frozen pilot")
    );
}

fn entry_fixture_with_adapter() -> InventoryEntry {
    let mut entry = entry_fixture();
    entry.source_adapter = Some(adapter_fixture());
    entry
}

fn entry_fixture() -> InventoryEntry {
    InventoryEntry {
        id: "realimpact-glass-goblet-row-0000".to_owned(),
        partition: Partition::Dev,
        recording_kind: RecordingKind::ControlledRealForceDeconvolvedTransfer,
        domain_id: "rigid-impact-glass-vessel".to_owned(),
        material_family: "glass".to_owned(),
        object_family_id: "glass-goblet".to_owned(),
        object_id: "realimpact-94-glassgoblet".to_owned(),
        source_id: "realimpact-fca2bd6cbb7e-archive-2023-04-10".to_owned(),
        generator_revision: None,
        mutation_parent_entry_id: None,
        geometry_revision: "mesh-cfd3b018f534-v1".to_owned(),
        support_condition: "thread-mesh-unversioned-in-archive".to_owned(),
        excitation_method: "instrumented-hammer-force-deconvolved".to_owned(),
        impact_position_id: "mesh-vertex-12351".to_owned(),
        impact_position_metres: [-0.007_681_29, -0.033_108_16, -0.002_606_2],
        listener_condition_id: "angle-000-distance-0230mm-mic-00".to_owned(),
        listener_position_metres: [0.23, -0.043_45, -0.91],
        sample_rate_hz: 48_000,
        sample_count: 4,
        audio_format: AudioFormat::F32LeMono,
        audio_payload: file_ref("audio.f32le", REALIMPACT_TRANSFER_SHA256),
        acquisition_metadata: file_ref("metadata.json", REALIMPACT_METADATA_SHA256),
        provenance_review: file_ref("provenance.md", REALIMPACT_PROVENANCE_SHA256),
        unavailable_components: REALIMPACT_UNAVAILABLE.map(str::to_owned).to_vec(),
        source_adapter: None,
        complete_acquisition: None,
    }
}

fn adapter_fixture() -> SourceAdapterProfile {
    SourceAdapterProfile::RealImpactForceDeconvolvedTransferV1 {
        repository_revision: REALIMPACT_REPOSITORY_REVISION.to_owned(),
        dataset_object_id: "94_GlassGoblet".to_owned(),
        row_index: 0,
        repository_readme: file_ref("README.md", REALIMPACT_README_SHA256),
        repository_license: file_ref("LICENSE", REALIMPACT_LICENSE_SHA256),
        preprocess_measurements: file_ref(
            "preprocess_measurements.py",
            REALIMPACT_MEASUREMENTS_SHA256,
        ),
        preprocess_annotations: file_ref(
            "preprocess_annotations.py",
            REALIMPACT_ANNOTATIONS_SHA256,
        ),
        dataset_download_script: file_ref("download.sh", REALIMPACT_DOWNLOAD_SHA256),
    }
}

fn file_ref(path: &str, sha256: &str) -> FileRef {
    FileRef {
        path: path.to_owned(),
        sha256: sha256.to_owned(),
    }
}

fn metadata_fixture() -> RealImpactMetadata {
    RealImpactMetadata {
        schema: "nextengine.experimental-realimpact-bounded-import.metadata.v1".to_owned(),
        status: "development_pilot_partial_source".to_owned(),
        retrieved_at: "2026-08-27".to_owned(),
        archive: ArchiveMetadata {
            url: format!("{REALIMPACT_ARCHIVE_PREFIX}94_GlassGoblet.zip"),
            content_length: 1,
            etag: "archive-etag".to_owned(),
            last_modified: "2023-04-10T10:29:20Z".to_owned(),
            central_directory_sha256: "a".repeat(64),
        },
        object: ObjectMetadata {
            dataset_object_id: "94_GlassGoblet".to_owned(),
            material_label: "glass".to_owned(),
            mesh_entry: "94_GlassGoblet/preprocessed/transformed.obj".to_owned(),
            mesh_sha256: "cfd3b018f534e41be719d12fff7bed27adf6b85f4a03bd55d34630e0bdebdf8f"
                .to_owned(),
            mesh_vertex_count: 48_036,
            mesh_bbox_min_metres: [-0.05, -0.05, -0.01],
            mesh_bbox_max_metres: [0.05, 0.05, 0.16],
        },
        acquisition: AcquisitionMetadata {
            sample_rate_hz: 48_000,
            support_condition: "thread-mesh-described-in-realimpact-paper-unversioned-in-archive"
                .to_owned(),
            excitation_method: "instrumented-impact-hammer-force-deconvolved-transfer".to_owned(),
            impact_vertex_count: 1,
            listener_position_count: 1,
            azimuth_degrees: vec![0],
            gantry_distance_offsets_millimetres: vec![0],
            paper_listener_distances_metres: vec![0.23],
            microphone_ids: vec![0],
        },
        downloaded_npy_hashes: DownloadedNpyHashes {
            angle_npy: "4".repeat(64),
            distance_npy: "5".repeat(64),
            listener_xyz_npy: "6".repeat(64),
            mic_id_npy: "7".repeat(64),
            vertex_id_npy: "8".repeat(64),
            vertex_xyz_npy: "9".repeat(64),
        },
        audio_entry: AudioEntryMetadata {
            path: "94_GlassGoblet/preprocessed/deconvolved_0db.npy".to_owned(),
            zip_crc32: "abcdef01".to_owned(),
            compressed_bytes: 16,
            uncompressed_bytes: 16,
            dtype: "f32_le".to_owned(),
            shape: [1, 4],
        },
        pilot_row: PilotRowMetadata {
            row_index: 0,
            impact_vertex_id: 12_351,
            impact_position_metres: [-0.007_681_29, -0.033_108_16, -0.002_606_2],
            azimuth_degrees: 0,
            gantry_distance_offset_millimetres: 0,
            microphone_id: 0,
            listener_position_metres: [0.23, -0.043_45, -0.91],
            sample_count: 4,
            duration_seconds: 4.0 / 48_000.0,
            raw_f32le_sha256: REALIMPACT_TRANSFER_SHA256.to_owned(),
            peak_abs: 0.5,
            rms: 0.279_508_497_187_473_7,
            normalized_audition_wav_sha256: "b".repeat(64),
        },
        unavailable_components: REALIMPACT_UNAVAILABLE.map(str::to_owned).to_vec(),
    }
}
