use super::*;

pub(super) fn normalize_source(
    source: &SourceReport,
    partition: Partition,
) -> Result<(String, String, Vec<IdentifiedRecording>), String> {
    let Some(AdapterEvidenceReport::SoundpacksGlassRecordingsIdentifiedRecordingV1 {
        object_id,
        material_label,
        recordings,
        ..
    }) = &source.adapter_evidence
    else {
        return Err(format!(
            "source {} has no matching typed SoundPacks E3 adapter evidence",
            source.id
        ));
    };
    if recordings.len() < 2 {
        return Err(format!(
            "source {} has fewer than two identified SoundPacks recordings",
            source.id
        ));
    }
    let archive = source
        .artifacts
        .iter()
        .find(|artifact| artifact.id == "audio-archive")
        .ok_or_else(|| format!("source {} is missing its SoundPacks archive", source.id))?;
    if archive.role != "audio_archive"
        || archive.cache_status != "CachedVerified"
        || archive.expected_byte_count.is_none()
        || archive.expected_sha256.is_none()
    {
        return Err(format!(
            "source {} has no hash-closed cached SoundPacks archive",
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
            corpus_role: CorpusRole::Unassigned,
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
