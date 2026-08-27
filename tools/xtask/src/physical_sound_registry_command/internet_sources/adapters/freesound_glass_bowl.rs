use std::collections::BTreeSet;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{
    AdapterAudit, AdapterEvidenceReport, AdapterProfile, ArtifactRole, EvidenceCapability,
    InternetSource, RecordingReport, RemoteArtifact, mp3,
};
use crate::physical_sound_registry_command::internet_sources::{
    FetchNormalizationPolicy, RedistributionPolicy,
};

pub(super) const ADAPTER_ID: &str = "freesound-glass-bowl-identified-recording-v1";

const SOURCE_ID: &str = "freesound-pack-14905-medium-glass-bowl";
const PUBLISHER_ID: &str = "freesound-user-ascap";
const PROJECT_ID: &str = "pack-14905-medium-glass-bowl";
const DECLARED_REVISION: &str = "review-2026-08-28";
const LANDING_PAGE_URL: &str = "https://freesound.org/people/ascap/packs/14905/";
const TERMS_URL: &str = "https://creativecommons.org/licenses/by-nc/4.0/";
const LICENSE_EXPRESSION: &str = "CC-BY-NC-4.0";
const PACK_ID: &str = "14905";
const OBJECT_ID: &str = "pack-14905-medium-glass-bowl";
const OBJECT_NAME: &str = "Medium-pitched glass bowl";
const MATERIAL_LABEL: &str = "Glass";
const AUTHOR_ID: &str = "4420518";
const PACK_IDENTITY_BYTES: u64 = 4_245;
const PACK_IDENTITY_SHA256: &str =
    "0b69f458441c57f301372b05b70d03ea17f7be494e85faba4db0bbd3ec60c59b";
const MAXIMUM_PACK_PAGE_BYTES: u64 = 512 * 1024;
const MAXIMUM_PREVIEW_BYTES: u64 = 256 * 1024;
const MAXIMUM_SAMPLE_FRAMES: u64 = 6 * 44_100;
const PACK_DESCRIPTION: &str = "This pack contains the sounds of a medium-pitched glass bowl being struck by a soft mallet, a fingertip, a piece of wood, and a piece of metal. Each file is numbered more or less according to brightness/intensity, with higher numbers signifying a brighter sound/more intense attack. If they are not perfectly ordered, then I give you permission to type at me in all caps. Comment what you are using the sound for!";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::physical_sound_registry_command::internet_sources) struct RecordingProfile {
    pub(super) recording_id: String,
    pub(super) sound_id: String,
}

#[derive(Serialize)]
pub(in crate::physical_sound_registry_command::internet_sources) struct RecordingEvidenceReport {
    pub(in crate::physical_sound_registry_command::internet_sources) sound_id: String,
    pub(in crate::physical_sound_registry_command::internet_sources) source_file_name: &'static str,
    pub(in crate::physical_sound_registry_command::internet_sources) preview_variant: &'static str,
    #[serde(flatten)]
    pub(in crate::physical_sound_registry_command::internet_sources) audio: RecordingReport,
}

#[derive(Clone, Copy)]
struct FrozenRecording {
    recording_id: &'static str,
    sound_id: &'static str,
    source_file_name: &'static str,
    duration_seconds: &'static str,
    expected_sample_frames: u64,
    preview_url: &'static str,
    expected_bytes: u64,
    sha256: &'static str,
}

const RECORDINGS: [FrozenRecording; 8] = [
    FrozenRecording {
        recording_id: "001",
        sound_id: "242460",
        source_file_name: "wood hit medium glass bowl 1.mp3",
        duration_seconds: "4.0",
        expected_sample_frames: 176_400,
        preview_url: "https://cdn.freesound.org/previews/242/242460_4420518-hq.mp3",
        expected_bytes: 73_540,
        sha256: "32beb1ac5f58c9a57ff3f57b866443cd404eefc4d995ae298d1b4e58cc51d60c",
    },
    FrozenRecording {
        recording_id: "002",
        sound_id: "242459",
        source_file_name: "wood hit medium glass bowl 2.mp3",
        duration_seconds: "4.0",
        expected_sample_frames: 176_400,
        preview_url: "https://cdn.freesound.org/previews/242/242459_4420518-hq.mp3",
        expected_bytes: 71_373,
        sha256: "038270b6281ce64cdd727a159fc4f6e7b65ab9b15b6b90fbb35c4de1b1f29362",
    },
    FrozenRecording {
        recording_id: "003",
        sound_id: "242458",
        source_file_name: "wood hit medium glass bowl 3.mp3",
        duration_seconds: "4.0",
        expected_sample_frames: 176_400,
        preview_url: "https://cdn.freesound.org/previews/242/242458_4420518-hq.mp3",
        expected_bytes: 82_189,
        sha256: "8f2be53ee61fd13475a0c63db4ae13ae2886f0efec9b6b13bece65809cd75a32",
    },
    FrozenRecording {
        recording_id: "004",
        sound_id: "242457",
        source_file_name: "wood hit medium glass bowl 4.mp3",
        duration_seconds: "5.0",
        expected_sample_frames: 220_500,
        preview_url: "https://cdn.freesound.org/previews/242/242457_4420518-hq.mp3",
        expected_bytes: 88_594,
        sha256: "7726eb1a41b6fbeb88fd3fe4c177df103d1cb41b947dedef631fe5840b04a00d",
    },
    FrozenRecording {
        recording_id: "005",
        sound_id: "242464",
        source_file_name: "wood hit medium glass bowl 5.mp3",
        duration_seconds: "5.0",
        expected_sample_frames: 220_500,
        preview_url: "https://cdn.freesound.org/previews/242/242464_4420518-hq.mp3",
        expected_bytes: 86_005,
        sha256: "b25c39a7a0d138e8ac50f7f74e5453e96d1c10ceeaaa6079462be363ea7b3468",
    },
    FrozenRecording {
        recording_id: "006",
        sound_id: "242463",
        source_file_name: "wood hit medium glass bowl 6.mp3",
        duration_seconds: "4.0",
        expected_sample_frames: 176_400,
        preview_url: "https://cdn.freesound.org/previews/242/242463_4420518-hq.mp3",
        expected_bytes: 80_985,
        sha256: "b29903528ba17353b870527e2104c68319221425c823c78a0188a086586a3e23",
    },
    FrozenRecording {
        recording_id: "007",
        sound_id: "242462",
        source_file_name: "wood hit medium glass bowl 7.mp3",
        duration_seconds: "4.0",
        expected_sample_frames: 176_400,
        preview_url: "https://cdn.freesound.org/previews/242/242462_4420518-hq.mp3",
        expected_bytes: 81_296,
        sha256: "61c9c0a36e386c472bc9c72c1eae2f3c396b4cb77070687609a09fdcc42c6af9",
    },
    FrozenRecording {
        recording_id: "008",
        sound_id: "242461",
        source_file_name: "wood hit medium glass bowl 8.mp3",
        duration_seconds: "3.5",
        expected_sample_frames: 154_350,
        preview_url: "https://cdn.freesound.org/previews/242/242461_4420518-hq.mp3",
        expected_bytes: 73_948,
        sha256: "511cc255b70b63d241b3ca978492a693f3ae117e3d7524ffb12eecf46e3bb11c",
    },
];

pub(super) fn validate_declaration(source: &InternetSource) -> Result<(), String> {
    let AdapterProfile::FreesoundGlassBowlIdentifiedRecordingV1 {
        pack_id,
        object_id,
        object_name,
        material_label,
        recordings,
    } = source
        .adapter_profile
        .as_ref()
        .ok_or_else(|| format!("source {} has no Freesound glass-bowl profile", source.id))?
    else {
        return Err(format!(
            "source {} has the wrong Freesound glass-bowl profile",
            source.id
        ));
    };
    if source.id != SOURCE_ID
        || source.publisher_id != PUBLISHER_ID
        || source.project_id != PROJECT_ID
        || source.declared_revision != DECLARED_REVISION
        || source.landing_page_url != LANDING_PAGE_URL
        || source.terms_url.as_deref() != Some(TERMS_URL)
        || source.license_expression != LICENSE_EXPRESSION
        || !matches!(
            source.redistribution_policy,
            RedistributionPolicy::ExternalResearchOnly
        )
        || pack_id != PACK_ID
        || object_id != OBJECT_ID
        || object_name != OBJECT_NAME
        || material_label != MATERIAL_LABEL
    {
        return Err(format!(
            "source {} does not match the frozen Freesound pack identity",
            source.id
        ));
    }
    if recordings.len() != RECORDINGS.len()
        || recordings.iter().zip(RECORDINGS).any(|(actual, expected)| {
            actual.recording_id != expected.recording_id || actual.sound_id != expected.sound_id
        })
    {
        return Err("Freesound glass-bowl recording identities changed".to_owned());
    }
    validate_artifacts(source)?;
    validate_capability_evidence(source)?;
    Ok(())
}

pub(super) fn audit(cache: &Path, source: &InternetSource) -> Result<AdapterAudit, String> {
    let pack_bytes = super::read_cached(cache, artifact(source, "pack-identity")?)?;
    let pack: PackIdentity = serde_json::from_slice(&pack_bytes)
        .map_err(|error| format!("parse canonical Freesound pack identity: {error}"))?;
    validate_pack_identity(&pack)?;
    let mut recordings = Vec::with_capacity(RECORDINGS.len());
    for expected in RECORDINGS {
        let bytes = super::read_cached(cache, artifact(source, &artifact_id(expected))?)?;
        let audio = mp3::validate_mpeg1_layer3(
            expected.recording_id,
            &bytes,
            mp3::Expectation {
                source_label: "Freesound HQ preview",
                sample_rate_hz: 44_100,
                channel_count: 2,
                maximum_sample_frames: MAXIMUM_SAMPLE_FRAMES,
            },
        )?;
        if audio.sample_frames != expected.expected_sample_frames {
            return Err(format!(
                "Freesound sound {} gapless duration changed",
                expected.sound_id
            ));
        }
        recordings.push(RecordingEvidenceReport {
            sound_id: expected.sound_id.to_owned(),
            source_file_name: expected.source_file_name,
            preview_variant: "public_hq_mp3",
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
        evidence: Some(
            AdapterEvidenceReport::FreesoundGlassBowlIdentifiedRecordingV1 {
                pack_id: PACK_ID.to_owned(),
                object_id: OBJECT_ID.to_owned(),
                object_name: OBJECT_NAME.to_owned(),
                material_label: MATERIAL_LABEL.to_owned(),
                recordings,
            },
        ),
    })
}

fn validate_artifacts(source: &InternetSource) -> Result<(), String> {
    if source.artifacts.len() != RECORDINGS.len() + 1 {
        return Err(format!(
            "source {} must contain eight previews and one pack identity",
            source.id
        ));
    }
    for expected in RECORDINGS {
        let artifact = artifact(source, &artifact_id(expected))?;
        if artifact.role != ArtifactRole::AudioPayload
            || artifact.url != expected.preview_url
            || artifact.redirect_policy.is_some()
            || artifact.normalization_policy.is_some()
            || artifact.maximum_bytes != MAXIMUM_PREVIEW_BYTES
            || artifact.expected_byte_count != Some(expected.expected_bytes)
            || artifact.expected_sha256.as_deref() != Some(expected.sha256)
        {
            return Err(format!(
                "Freesound preview artifact {} does not match adapter v1",
                artifact.id
            ));
        }
    }
    let pack = artifact(source, "pack-identity")?;
    if pack.role != ArtifactRole::ProjectDescription
        || pack.url != LANDING_PAGE_URL
        || pack.redirect_policy.is_some()
        || pack.normalization_policy != Some(FetchNormalizationPolicy::FreesoundPackIdentityV1)
        || pack.maximum_bytes != MAXIMUM_PACK_PAGE_BYTES
        || pack.expected_byte_count != Some(PACK_IDENTITY_BYTES)
        || pack.expected_sha256.as_deref() != Some(PACK_IDENTITY_SHA256)
    {
        return Err("Freesound pack identity artifact does not match adapter v1".to_owned());
    }
    Ok(())
}

fn validate_capability_evidence(source: &InternetSource) -> Result<(), String> {
    let artifact_ids = source
        .artifacts
        .iter()
        .map(|artifact| artifact.id.clone())
        .collect::<Vec<_>>();
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
            "source {} must bind every E3 capability to all frozen Freesound artifacts",
            source.id
        ));
    }
    Ok(())
}

fn validate_pack_identity(pack: &PackIdentity) -> Result<(), String> {
    if pack.schema != "nextengine.experimental-freesound-pack-identity.v1"
        || pack.source_url != LANDING_PAGE_URL
        || pack.author != "ascap"
        || pack.author_id != AUTHOR_ID
        || pack.pack_id != PACK_ID
        || pack.title != "medium-pitched glass bowl"
        || pack.description != PACK_DESCRIPTION
        || pack.sounds.len() != 12
    {
        return Err("canonical Freesound pack identity changed".to_owned());
    }
    for expected in RECORDINGS {
        let sound = pack
            .sounds
            .iter()
            .find(|sound| sound.sound_id == expected.sound_id)
            .ok_or_else(|| format!("Freesound pack is missing sound {}", expected.sound_id))?;
        let expected_lq = expected.preview_url.replace("-hq.mp3", "-lq.mp3");
        if sound.title != expected.source_file_name
            || sound.duration_seconds != expected.duration_seconds
            || sound.sample_rate_hz != "44100.0"
            || sound.lq_mp3_url != expected_lq
            || sound.license_label != "Attribution NonCommercial"
        {
            return Err(format!(
                "Freesound pack metadata changed for sound {}",
                expected.sound_id
            ));
        }
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
                "source {} is missing Freesound glass-bowl artifact {id}",
                source.id
            )
        })
}

fn artifact_id(recording: FrozenRecording) -> String {
    format!("impact-{}", recording.recording_id)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PackIdentity {
    schema: String,
    source_url: String,
    author: String,
    author_id: String,
    pack_id: String,
    title: String,
    description: String,
    sounds: Vec<SoundIdentity>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SoundIdentity {
    sound_id: String,
    title: String,
    duration_seconds: String,
    sample_rate_hz: String,
    lq_mp3_url: String,
    license_label: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physical_sound_registry_command::FileRef;
    use crate::physical_sound_registry_command::internet_sources::{
        CapabilityEvidence, RemoteArtifact,
    };

    #[test]
    fn declaration_requires_exact_pack_preview_and_policy_identity() {
        let mut source = test_source();
        validate_declaration(&source).expect("frozen Freesound declaration");
        source.artifacts[8].normalization_policy = None;
        assert!(validate_declaration(&source).is_err());
    }

    fn test_source() -> InternetSource {
        let mut artifacts = RECORDINGS
            .into_iter()
            .map(|recording| RemoteArtifact {
                id: artifact_id(recording),
                role: ArtifactRole::AudioPayload,
                url: recording.preview_url.to_owned(),
                redirect_policy: None,
                normalization_policy: None,
                maximum_bytes: MAXIMUM_PREVIEW_BYTES,
                expected_byte_count: Some(recording.expected_bytes),
                expected_sha256: Some(recording.sha256.to_owned()),
            })
            .collect::<Vec<_>>();
        artifacts.push(RemoteArtifact {
            id: "pack-identity".to_owned(),
            role: ArtifactRole::ProjectDescription,
            url: LANDING_PAGE_URL.to_owned(),
            redirect_policy: None,
            normalization_policy: Some(FetchNormalizationPolicy::FreesoundPackIdentityV1),
            maximum_bytes: MAXIMUM_PACK_PAGE_BYTES,
            expected_byte_count: Some(PACK_IDENTITY_BYTES),
            expected_sha256: Some(PACK_IDENTITY_SHA256.to_owned()),
        });
        let artifact_ids = artifacts
            .iter()
            .map(|artifact| artifact.id.clone())
            .collect::<Vec<_>>();
        InternetSource {
            id: SOURCE_ID.to_owned(),
            publisher_id: PUBLISHER_ID.to_owned(),
            project_id: PROJECT_ID.to_owned(),
            declared_revision: DECLARED_REVISION.to_owned(),
            review_date: "2026-08-28".to_owned(),
            landing_page_url: LANDING_PAGE_URL.to_owned(),
            terms_url: Some(TERMS_URL.to_owned()),
            adapter_id: ADAPTER_ID.to_owned(),
            adapter_profile: Some(AdapterProfile::FreesoundGlassBowlIdentifiedRecordingV1 {
                pack_id: PACK_ID.to_owned(),
                object_id: OBJECT_ID.to_owned(),
                object_name: OBJECT_NAME.to_owned(),
                material_label: MATERIAL_LABEL.to_owned(),
                recordings: RECORDINGS
                    .into_iter()
                    .map(|recording| RecordingProfile {
                        recording_id: recording.recording_id.to_owned(),
                        sound_id: recording.sound_id.to_owned(),
                    })
                    .collect(),
            }),
            license_expression: LICENSE_EXPRESSION.to_owned(),
            redistribution_policy: RedistributionPolicy::ExternalResearchOnly,
            provenance_review: FileRef {
                path: "freesound-provenance.md".to_owned(),
                sha256: "aa".repeat(32),
            },
            artifacts,
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
