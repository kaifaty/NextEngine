use super::*;
use crate::physical_sound_registry_command::ArtifactReport;
use crate::physical_sound_registry_command::internet_sources::adapters::ycb_impact::RecordingEvidenceReport;
use crate::physical_sound_registry_command::internet_sources::adapters::{
    AdapterEvidenceReport, RecordingReport,
};
use crate::physical_sound_registry_command::internet_sources::{
    CapabilityReport, RemoteArtifactReport, SourceReport,
};

#[test]
fn normalizes_ycb_source_without_treating_upstream_test_as_holdout() {
    let source = test_ycb_source();
    let (source_group, object_group, entries) =
        normalize_source(&source, Partition::Dev).expect("normalize YCB E3 source");
    assert_eq!(
        source_group,
        "iri-csic-upc-ctu--ycb-impact-sounds--osf-bj5w8-2022-09-27"
    );
    assert_eq!(
        object_group,
        "iri-csic-upc-ctu--ycb-impact-sounds--osf-bj5w8-2022-09-27--object-28"
    );
    assert_eq!(entries.len(), 2);
    assert!(entries.iter().all(|entry| entry.partition == "dev"));
    assert!(entries.iter().all(|entry| entry.material_label == "Glass"));
    assert_eq!(entries[0].recording_id, "horizontal-0_14-001");
    assert_eq!(entries[0].sample_rate_hz, 48_000);
    assert_eq!(entries[0].channel_count, 2);
}

fn test_ycb_source() -> SourceReport {
    let recording_ids = ["horizontal-0_14-001", "horizontal-0_25-001"];
    SourceReport {
        id: "ycb-impact-object-028-skillet-lid".to_owned(),
        publisher_id: "iri-csic-upc-ctu".to_owned(),
        project_id: "ycb-impact-sounds".to_owned(),
        declared_revision: "osf-bj5w8-2022-09-27".to_owned(),
        review_date: "2026-08-27".to_owned(),
        landing_page_url: "https://osf.io/4tcp6/".to_owned(),
        terms_url: None,
        adapter_id: "ycb-impact-identified-recording-v1".to_owned(),
        adapter_evidence: Some(AdapterEvidenceReport::YcbImpactIdentifiedRecordingV1 {
            object_id: "28".to_owned(),
            object_name: "Skillet lid".to_owned(),
            primary_material_label: "Glass".to_owned(),
            secondary_material_label: Some("Hard Plastic".to_owned()),
            source_split: "test".to_owned(),
            recordings: recording_ids
                .iter()
                .enumerate()
                .map(|(index, recording_id)| RecordingEvidenceReport {
                    condition_id: if index == 0 {
                        "horizontal-0_14"
                    } else {
                        "horizontal-0_25"
                    }
                    .to_owned(),
                    clip_id: "001".to_owned(),
                    osf_file_id: format!("{:024x}", index + 1),
                    source_file_name: "Clip_1.ogg",
                    container_format: "riff_wave",
                    audio: RecordingReport {
                        recording_id: (*recording_id).to_owned(),
                        sample_encoding: "ieee_float32_le",
                        sample_rate_hz: 48_000,
                        channel_count: 2,
                        bits_per_sample: 32,
                        sample_frames: 240_000,
                    },
                })
                .collect(),
        }),
        license_expression: "NOASSERTION".to_owned(),
        redistribution_policy: "external_research_only",
        provenance_review: ArtifactReport {
            sha256: "dd".repeat(32),
            byte_count: 2_083,
        },
        artifacts: recording_ids
            .iter()
            .map(|recording_id| RemoteArtifactReport {
                id: format!("impact-{recording_id}"),
                role: "audio_payload",
                url: format!("https://example.invalid/{recording_id}"),
                maximum_bytes: 2_097_152,
                expected_byte_count: Some(1_920_058),
                expected_sha256: Some("ab".repeat(32)),
                cache_status: "CachedVerified",
            })
            .collect(),
        capabilities: REQUIRED_CAPABILITIES
            .iter()
            .map(|capability| CapabilityReport {
                capability,
                artifact_ids: recording_ids
                    .iter()
                    .map(|recording_id| format!("impact-{recording_id}"))
                    .collect(),
                bytes_available: true,
                adapter_validated: true,
                available: true,
            })
            .collect(),
        supported_tiers: vec!["E3IdentifiedRecording"],
        source_status: "EvidenceReady",
    }
}
