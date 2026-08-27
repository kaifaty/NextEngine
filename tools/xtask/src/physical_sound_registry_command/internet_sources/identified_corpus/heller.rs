use super::*;

pub(super) fn normalize_source(
    source: &SourceReport,
    partition: Partition,
) -> Result<(String, String, Vec<IdentifiedRecording>), String> {
    let Some(AdapterEvidenceReport::HellerImpactIdentifiedRecordingV1 {
        object_id,
        material_label,
        recordings,
        ..
    }) = &source.adapter_evidence
    else {
        return Err(format!(
            "source {} has no matching typed Heller E3 adapter evidence",
            source.id
        ));
    };
    if recordings.is_empty() {
        return Err(format!("source {} has no identified recordings", source.id));
    }
    let archive = source
        .artifacts
        .iter()
        .find(|artifact| artifact.id == "impact-audio-archive")
        .ok_or_else(|| {
            format!(
                "source {} is missing its identified audio archive",
                source.id
            )
        })?;
    if archive.role != "audio_archive"
        || archive.cache_status != "CachedVerified"
        || archive.expected_byte_count.is_none()
        || archive.expected_sha256.is_none()
    {
        return Err(format!(
            "source {} has no hash-closed cached audio archive",
            source.id
        ));
    }
    let source_group_id = format!(
        "{}--{}--{}",
        source.publisher_id, source.project_id, source.declared_revision
    );
    let object_group_id = format!("{source_group_id}--object-{object_id}");
    let entries = recordings
        .iter()
        .map(|recording| IdentifiedRecording {
            entry_id: format!("{}--impact-{}", source.id, recording.audio.recording_id),
            partition: partition.as_str(),
            evidence_tier: "E3IdentifiedRecording",
            publisher_id: source.publisher_id.clone(),
            project_id: source.project_id.clone(),
            declared_revision: source.declared_revision.clone(),
            source_id: source.id.clone(),
            source_group_id: source_group_id.clone(),
            object_group_id: object_group_id.clone(),
            object_id: object_id.clone(),
            material_label: material_label.clone(),
            recording_id: recording.audio.recording_id.clone(),
            sample_encoding: recording.audio.sample_encoding,
            sample_rate_hz: recording.audio.sample_rate_hz,
            channel_count: recording.audio.channel_count,
            bits_per_sample: recording.audio.bits_per_sample,
            sample_frames: recording.audio.sample_frames,
            audio_file_bytes: recording.audio_file_bytes,
            audio_file_sha256: recording.audio_file_sha256.to_owned(),
            license_expression: source.license_expression.clone(),
            redistribution_policy: source.redistribution_policy,
        })
        .collect();
    Ok((source_group_id, object_group_id, entries))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physical_sound_registry_command::ArtifactReport;
    use crate::physical_sound_registry_command::internet_sources::adapters::RecordingReport;
    use crate::physical_sound_registry_command::internet_sources::adapters::heller_impact::RecordingEvidenceReport;
    use crate::physical_sound_registry_command::internet_sources::{
        CapabilityReport, RemoteArtifactReport,
    };

    #[test]
    fn normalizes_archive_backed_recordings_without_promoting_the_archive_hash() {
        let source = SourceReport {
            id: "cmu-auditorylab-impact-glass-vase".to_owned(),
            publisher_id: "carnegie-mellon-university-auditorylab".to_owned(),
            project_id: "sound-events-database-impact-events".to_owned(),
            declared_revision: "figshare-20205035-v1-2022-07-07".to_owned(),
            review_date: "2026-08-27".to_owned(),
            landing_page_url: "https://kilthub.cmu.edu/articles/media/Impact_Events/20205035"
                .to_owned(),
            terms_url: None,
            adapter_id: "heller-impact-identified-recording-v1".to_owned(),
            adapter_evidence: Some(AdapterEvidenceReport::HellerImpactIdentifiedRecordingV1 {
                object_id: "glass-vase".to_owned(),
                object_name: "Glass vase".to_owned(),
                material_label: "Glass".to_owned(),
                event_label: "Marbles Dropped in Glass Vase".to_owned(),
                recordings: vec![RecordingEvidenceReport {
                    repeat_id: "001".to_owned(),
                    archive_entry: "Impacts_audio1/glass.wav",
                    audio_file_bytes: 44,
                    audio_file_sha256: "abababababababababababababababababababababababababababababababab",
                    audio: RecordingReport {
                        recording_id: "001".to_owned(),
                        sample_encoding: "pcm_s16_le",
                        sample_rate_hz: 44_100,
                        channel_count: 2,
                        bits_per_sample: 16,
                        sample_frames: 2,
                    },
                }],
            }),
            license_expression: "LicenseRef-Heller-Sound-Events-Research-Only".to_owned(),
            redistribution_policy: "external_research_only",
            provenance_review: ArtifactReport {
                sha256: "cd".repeat(32),
                byte_count: 1_024,
            },
            artifacts: vec![RemoteArtifactReport {
                id: "impact-audio-archive".to_owned(),
                role: "audio_archive",
                url: "https://ndownloader.figshare.com/files/36113411".to_owned(),
                maximum_bytes: 40 * 1024 * 1024,
                expected_byte_count: Some(38_059_995),
                expected_sha256: Some("ef".repeat(32)),
                cache_status: "CachedVerified",
            }],
            capabilities: REQUIRED_CAPABILITIES
                .iter()
                .map(|capability| CapabilityReport {
                    capability,
                    artifact_ids: vec!["impact-audio-archive".to_owned()],
                    bytes_available: true,
                    adapter_validated: true,
                    available: true,
                })
                .collect(),
            supported_tiers: vec!["E3IdentifiedRecording"],
            source_status: "EvidenceReady",
        };
        let (_, object_group, entries) =
            super::normalize_source(&source, Partition::Dev).expect("normalize Heller source");
        assert!(object_group.ends_with("--object-glass-vase"));
        assert_eq!(entries[0].audio_file_bytes, 44);
        assert_eq!(entries[0].audio_file_sha256, "ab".repeat(32));
        assert_eq!(entries[0].material_label, "Glass");
    }
}
