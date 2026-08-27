use std::collections::BTreeSet;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{
    AdapterAudit, AdapterEvidenceReport, AdapterProfile, ArtifactRole, EvidenceCapability,
    InternetSource, RecordingReport, RemoteArtifact, wav, zip_archive,
};
use crate::physical_sound_registry_command::internet_sources::{
    FetchRedirectPolicy, RedistributionPolicy,
};

pub(super) const ADAPTER_ID: &str = "heller-impact-identified-recording-v1";

const SOURCE_ID: &str = "cmu-auditorylab-impact-glass-vase";
const PUBLISHER_ID: &str = "carnegie-mellon-university-auditorylab";
const PROJECT_ID: &str = "sound-events-database-impact-events";
const DECLARED_REVISION: &str = "figshare-20205035-v1-2022-07-07";
const LANDING_PAGE_URL: &str = "https://kilthub.cmu.edu/articles/media/Impact_Events/20205035";
const LICENSE_EXPRESSION: &str = "LicenseRef-Heller-Sound-Events-Research-Only";
const OBJECT_ID: &str = "glass-vase";
const OBJECT_NAME: &str = "Glass vase";
const MATERIAL_LABEL: &str = "Glass";
const EVENT_LABEL: &str = "Marbles Dropped in Glass Vase";
const AUDIO_ARCHIVE_URL: &str = "https://ndownloader.figshare.com/files/36113411";
const AUDIO_ARCHIVE_BYTES: u64 = 38_059_995;
const AUDIO_ARCHIVE_MAXIMUM_BYTES: u64 = 40 * 1024 * 1024;
const AUDIO_ARCHIVE_SHA256: &str =
    "1d57964b4f48d277b6cf567a0bce938b3b8901f6b43ca6787db1036e8d737783";
const RECORDING_NOTES_URL: &str = "https://ndownloader.figshare.com/files/36113402";
const RECORDING_NOTES_BYTES: u64 = 42_010;
const RECORDING_NOTES_MAXIMUM_BYTES: u64 = 64 * 1024;
const RECORDING_NOTES_SHA256: &str =
    "c38fa7cc1d49534e6979609d6019354efe040b52dcadf667e8f619c9f2dcf8a1";
const MAX_RECORDING_FRAMES: u64 = 750_000;
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::physical_sound_registry_command::internet_sources) struct RecordingProfile {
    pub(super) repeat_id: String,
    pub(super) archive_entry: String,
}

#[derive(Serialize)]
pub(in crate::physical_sound_registry_command::internet_sources) struct RecordingEvidenceReport {
    pub(in crate::physical_sound_registry_command::internet_sources) repeat_id: String,
    pub(in crate::physical_sound_registry_command::internet_sources) archive_entry: &'static str,
    pub(in crate::physical_sound_registry_command::internet_sources) audio_file_bytes: u64,
    pub(in crate::physical_sound_registry_command::internet_sources) audio_file_sha256:
        &'static str,
    #[serde(flatten)]
    pub(in crate::physical_sound_registry_command::internet_sources) audio: RecordingReport,
}

#[derive(Clone, Copy)]
struct FrozenRecording {
    repeat_id: &'static str,
    archive_entry: &'static str,
    bytes: u64,
    sha256: &'static str,
    sample_frames: u64,
}

const RECORDINGS: [FrozenRecording; 5] = [
    FrozenRecording {
        repeat_id: "001",
        archive_entry: "Impacts_audio1/Marbles Dropped in Glass Vase/marblesbouncinginvaise1.wav",
        bytes: 2_876_540,
        sha256: "67d9f83b35b77f54377d0c3b0770b865ed749f20d1f0eed19685e6dcdf7830cd",
        sample_frames: 719_124,
    },
    FrozenRecording {
        repeat_id: "002",
        archive_entry: "Impacts_audio1/Marbles Dropped in Glass Vase/marblesbouncinginvaise2.wav",
        bytes: 2_074_508,
        sha256: "adf7ddf934ea7bcb269cf69c89393913aaa7693aba15e103a74b1c47d0442243",
        sample_frames: 518_616,
    },
    FrozenRecording {
        repeat_id: "003",
        archive_entry: "Impacts_audio1/Marbles Dropped in Glass Vase/marblesbouncinginvaise3.wav",
        bytes: 2_421_164,
        sha256: "d57b75c672440fb022a7015a6535e959660ba56cdee1b33acbfac8d7ee518587",
        sample_frames: 605_280,
    },
    FrozenRecording {
        repeat_id: "004",
        archive_entry: "Impacts_audio1/Marbles Dropped in Glass Vase/marblesbouncinginvaise4.wav",
        bytes: 2_359_340,
        sha256: "0050a1e6cd76491cfd3a47d90321e10f949cb3a0124de8e8d2de8124e04370dd",
        sample_frames: 589_824,
    },
    FrozenRecording {
        repeat_id: "005",
        archive_entry: "Impacts_audio1/Marbles Dropped in Glass Vase/marblesbouncinginvaise5.wav",
        bytes: 1_957_932,
        sha256: "ae9d833054d0ae89a00fb5530a82d045d2ab9e91e7749304fccb69a42745bc6d",
        sample_frames: 489_472,
    },
];

pub(super) fn validate_declaration(source: &InternetSource) -> Result<(), String> {
    let AdapterProfile::HellerImpactIdentifiedRecordingV1 {
        object_id,
        object_name,
        material_label,
        event_label,
        recordings,
    } = source
        .adapter_profile
        .as_ref()
        .ok_or_else(|| format!("source {} has no Heller Impact profile", source.id))?
    else {
        return Err(format!(
            "source {} has the wrong Heller Impact profile",
            source.id
        ));
    };
    if source.id != SOURCE_ID
        || source.publisher_id != PUBLISHER_ID
        || source.project_id != PROJECT_ID
        || source.declared_revision != DECLARED_REVISION
        || source.landing_page_url != LANDING_PAGE_URL
        || source.terms_url.as_deref() != Some(LANDING_PAGE_URL)
        || source.license_expression != LICENSE_EXPRESSION
        || !matches!(
            source.redistribution_policy,
            RedistributionPolicy::ExternalResearchOnly
        )
        || object_id != OBJECT_ID
        || object_name != OBJECT_NAME
        || material_label != MATERIAL_LABEL
        || event_label != EVENT_LABEL
    {
        return Err(format!(
            "source {} does not match the frozen official Heller Impact identity",
            source.id
        ));
    }
    validate_recording_profile(recordings)?;
    validate_artifacts(source)?;
    validate_capability_evidence(source)?;
    Ok(())
}

pub(super) fn audit(cache: &Path, source: &InternetSource) -> Result<AdapterAudit, String> {
    let AdapterProfile::HellerImpactIdentifiedRecordingV1 {
        object_id,
        object_name,
        material_label,
        event_label,
        ..
    } = source
        .adapter_profile
        .as_ref()
        .ok_or_else(|| format!("source {} has no Heller Impact profile", source.id))?
    else {
        return Err(format!(
            "source {} has the wrong Heller Impact profile",
            source.id
        ));
    };
    let notes = super::read_cached(cache, artifact(source, "recording-notes")?)?;
    validate_recording_notes(&notes)?;
    let archive = super::read_cached(cache, artifact(source, "impact-audio-archive")?)?;
    let mut recordings = Vec::with_capacity(RECORDINGS.len());
    for expected in RECORDINGS {
        let bytes = zip_archive::read_exact_entry(
            &archive,
            expected.archive_entry,
            expected.bytes,
            expected.sha256,
        )?;
        let audio = wav::validate_pcm16_wav(
            expected.repeat_id,
            &bytes,
            wav::Pcm16WavExpectation {
                source_label: "Heller Impact recording",
                sample_rate_hz: 44_100,
                channel_count: 2,
                maximum_frames: MAX_RECORDING_FRAMES,
            },
        )?;
        if audio.sample_frames != expected.sample_frames {
            return Err(format!(
                "Heller Impact recording {} must contain exactly {} frames",
                expected.repeat_id, expected.sample_frames
            ));
        }
        recordings.push(RecordingEvidenceReport {
            repeat_id: expected.repeat_id.to_owned(),
            archive_entry: expected.archive_entry,
            audio_file_bytes: expected.bytes,
            audio_file_sha256: expected.sha256,
            audio,
        });
    }
    Ok(AdapterAudit {
        validated_capabilities: BTreeSet::from([
            EvidenceCapability::MaterialIdentity,
            EvidenceCapability::ObjectIdentity,
            EvidenceCapability::RealRecording,
            EvidenceCapability::RepeatIdentity,
        ]),
        evidence: Some(AdapterEvidenceReport::HellerImpactIdentifiedRecordingV1 {
            object_id: object_id.clone(),
            object_name: object_name.clone(),
            material_label: material_label.clone(),
            event_label: event_label.clone(),
            recordings,
        }),
    })
}

fn validate_recording_profile(recordings: &[RecordingProfile]) -> Result<(), String> {
    if recordings.len() != RECORDINGS.len()
        || recordings.iter().zip(RECORDINGS).any(|(actual, expected)| {
            actual.repeat_id != expected.repeat_id || actual.archive_entry != expected.archive_entry
        })
    {
        return Err("Heller Impact recording identities do not match adapter v1".to_owned());
    }
    Ok(())
}

fn validate_artifacts(source: &InternetSource) -> Result<(), String> {
    if source.artifacts.len() != 2 {
        return Err(format!(
            "source {} must contain one audio archive and recording notes",
            source.id
        ));
    }
    require_artifact(
        artifact(source, "impact-audio-archive")?,
        ArtifactRole::AudioArchive,
        AUDIO_ARCHIVE_URL,
        AUDIO_ARCHIVE_MAXIMUM_BYTES,
        AUDIO_ARCHIVE_BYTES,
        AUDIO_ARCHIVE_SHA256,
    )?;
    require_artifact(
        artifact(source, "recording-notes")?,
        ArtifactRole::Metadata,
        RECORDING_NOTES_URL,
        RECORDING_NOTES_MAXIMUM_BYTES,
        RECORDING_NOTES_BYTES,
        RECORDING_NOTES_SHA256,
    )?;
    Ok(())
}

fn validate_capability_evidence(source: &InternetSource) -> Result<(), String> {
    let artifact_ids = vec![
        "impact-audio-archive".to_owned(),
        "recording-notes".to_owned(),
    ];
    let required = [
        EvidenceCapability::MaterialIdentity,
        EvidenceCapability::ObjectIdentity,
        EvidenceCapability::RealRecording,
        EvidenceCapability::RepeatIdentity,
    ];
    if source.capability_evidence.len() != required.len()
        || source
            .capability_evidence
            .iter()
            .zip(required)
            .any(|(evidence, capability)| {
                evidence.capability != capability || evidence.artifact_ids != artifact_ids
            })
    {
        return Err(format!(
            "source {} must bind every E3 capability to the frozen Heller artifacts",
            source.id
        ));
    }
    Ok(())
}

fn artifact<'a>(source: &'a InternetSource, id: &str) -> Result<&'a RemoteArtifact, String> {
    source
        .artifacts
        .iter()
        .find(|artifact| artifact.id == id)
        .ok_or_else(|| {
            format!(
                "source {} is missing Heller Impact artifact {id}",
                source.id
            )
        })
}

fn require_artifact(
    artifact: &RemoteArtifact,
    role: ArtifactRole,
    url: &str,
    maximum_bytes: u64,
    expected_bytes: u64,
    expected_sha256: &str,
) -> Result<(), String> {
    if artifact.role != role
        || artifact.url != url
        || artifact.redirect_policy != Some(FetchRedirectPolicy::FigshareKiltHubV1)
        || artifact.maximum_bytes != maximum_bytes
        || artifact.expected_byte_count != Some(expected_bytes)
        || artifact.expected_sha256.as_deref() != Some(expected_sha256)
    {
        return Err(format!(
            "Heller Impact artifact {} does not match adapter v1",
            artifact.id
        ));
    }
    Ok(())
}

fn validate_recording_notes(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() as u64 != RECORDING_NOTES_BYTES
        || !bytes.starts_with(br"{\rtf1")
        || !contains_ascii(bytes, b"Recording Notes for Impacts")
        || !contains_ascii(bytes, b"Earthworks QTC30 Microphones")
    {
        return Err(
            "Heller Impact recording notes do not match the pinned RTF structure".to_owned(),
        );
    }
    Ok(())
}

fn contains_ascii(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physical_sound_registry_command::FileRef;
    use crate::physical_sound_registry_command::internet_sources::{
        CapabilityEvidence, RemoteArtifact,
    };

    #[test]
    fn declaration_freezes_one_explicit_glass_object_and_five_repeats() {
        let mut source = test_source();
        validate_declaration(&source).expect("frozen Heller declaration");
        let Some(AdapterProfile::HellerImpactIdentifiedRecordingV1 { material_label, .. }) =
            source.adapter_profile.as_mut()
        else {
            panic!("Heller profile");
        };
        *material_label = "Ceramic".to_owned();
        assert!(validate_declaration(&source).is_err());

        let mut source = test_source();
        source.artifacts[0].redirect_policy = None;
        assert!(validate_declaration(&source).is_err());
    }

    fn test_source() -> InternetSource {
        let artifact_ids = vec![
            "impact-audio-archive".to_owned(),
            "recording-notes".to_owned(),
        ];
        InternetSource {
            id: SOURCE_ID.to_owned(),
            publisher_id: PUBLISHER_ID.to_owned(),
            project_id: PROJECT_ID.to_owned(),
            declared_revision: DECLARED_REVISION.to_owned(),
            review_date: "2026-08-27".to_owned(),
            landing_page_url: LANDING_PAGE_URL.to_owned(),
            terms_url: Some(LANDING_PAGE_URL.to_owned()),
            adapter_id: ADAPTER_ID.to_owned(),
            adapter_profile: Some(AdapterProfile::HellerImpactIdentifiedRecordingV1 {
                object_id: OBJECT_ID.to_owned(),
                object_name: OBJECT_NAME.to_owned(),
                material_label: MATERIAL_LABEL.to_owned(),
                event_label: EVENT_LABEL.to_owned(),
                recordings: RECORDINGS
                    .iter()
                    .map(|recording| RecordingProfile {
                        repeat_id: recording.repeat_id.to_owned(),
                        archive_entry: recording.archive_entry.to_owned(),
                    })
                    .collect(),
            }),
            license_expression: LICENSE_EXPRESSION.to_owned(),
            redistribution_policy: RedistributionPolicy::ExternalResearchOnly,
            provenance_review: FileRef {
                path: "provenance.md".to_owned(),
                sha256: "aa".repeat(32),
            },
            artifacts: vec![
                RemoteArtifact {
                    id: "impact-audio-archive".to_owned(),
                    role: ArtifactRole::AudioArchive,
                    url: AUDIO_ARCHIVE_URL.to_owned(),
                    redirect_policy: Some(FetchRedirectPolicy::FigshareKiltHubV1),
                    normalization_policy: None,
                    maximum_bytes: AUDIO_ARCHIVE_MAXIMUM_BYTES,
                    expected_byte_count: Some(AUDIO_ARCHIVE_BYTES),
                    expected_sha256: Some(AUDIO_ARCHIVE_SHA256.to_owned()),
                },
                RemoteArtifact {
                    id: "recording-notes".to_owned(),
                    role: ArtifactRole::Metadata,
                    url: RECORDING_NOTES_URL.to_owned(),
                    redirect_policy: Some(FetchRedirectPolicy::FigshareKiltHubV1),
                    normalization_policy: None,
                    maximum_bytes: RECORDING_NOTES_MAXIMUM_BYTES,
                    expected_byte_count: Some(RECORDING_NOTES_BYTES),
                    expected_sha256: Some(RECORDING_NOTES_SHA256.to_owned()),
                },
            ],
            capability_evidence: [
                EvidenceCapability::MaterialIdentity,
                EvidenceCapability::ObjectIdentity,
                EvidenceCapability::RealRecording,
                EvidenceCapability::RepeatIdentity,
            ]
            .into_iter()
            .map(|capability| CapabilityEvidence {
                capability,
                artifact_ids: artifact_ids.clone(),
            })
            .collect(),
        }
    }
}
