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

pub(super) const ADAPTER_ID: &str = "freesound-wine-glass-identified-recording-v1";

const SOURCE_ID: &str = "freesound-pack-41981-wine-glass";
const PUBLISHER_ID: &str = "freesound-user-wasserbjorn";
const PROJECT_ID: &str = "pack-41981-metal-clank";
const DECLARED_REVISION: &str = "review-2026-08-28";
const LANDING_PAGE_URL: &str = "https://freesound.org/people/wasserbjorn/packs/41981/";
const TERMS_URL: &str = "https://creativecommons.org/publicdomain/zero/1.0/";
const LICENSE_EXPRESSION: &str = "CC0-1.0";
const PACK_ID: &str = "41981";
const OBJECT_ID: &str = "pack-41981-wine-glass";
const OBJECT_NAME: &str = "Wine glass struck by a knife";
const MATERIAL_LABEL: &str = "Glass";
const AUTHOR_ID: &str = "13431397";
const PACK_IDENTITY_BYTES: u64 = 3_673;
const PACK_IDENTITY_SHA256: &str =
    "e1eceb0d378fa1f243d2f869f32903b4dd9ac89831110dbdcde797118d14c559";
const MAXIMUM_PACK_PAGE_BYTES: u64 = 512 * 1024;
const MAXIMUM_PREVIEW_BYTES: u64 = 512 * 1024;
const MAXIMUM_SAMPLE_FRAMES: u64 = 30 * 44_100;
const PACK_DESCRIPTION: &str = "";

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

const RECORDINGS: [FrozenRecording; 3] = [
    FrozenRecording {
        recording_id: "001",
        sound_id: "761160",
        source_file_name: "Knife hits wine glass 1",
        duration_seconds: "14.3182",
        expected_sample_frames: 631_433,
        preview_url: "https://cdn.freesound.org/previews/761/761160_13431397-hq.mp3",
        expected_bytes: 260_587,
        sha256: "f477d0e2de75bdf81e3e2ec6b2e220b7faa0466412073d547237b13da7711085",
    },
    FrozenRecording {
        recording_id: "002",
        sound_id: "761161",
        source_file_name: "Knife hits wine glass 2",
        duration_seconds: "21.8182",
        expected_sample_frames: 962_183,
        preview_url: "https://cdn.freesound.org/previews/761/761161_13431397-hq.mp3",
        expected_bytes: 396_582,
        sha256: "4d371e9d9583796b524e664093aeedb295006da8a05f143cb7409ce8e7ee5991",
    },
    FrozenRecording {
        recording_id: "003",
        sound_id: "761162",
        source_file_name: "Knife hits wine glass 3",
        duration_seconds: "22.5",
        expected_sample_frames: 992_251,
        preview_url: "https://cdn.freesound.org/previews/761/761162_13431397-hq.mp3",
        expected_bytes: 409_997,
        sha256: "7be43b4a78f3a9775e62225cd3c96d1eeab1c3f975c8e4a6650d0e1c2a029065",
    },
];

pub(super) fn validate_declaration(source: &InternetSource) -> Result<(), String> {
    let AdapterProfile::FreesoundWineGlassIdentifiedRecordingV1 {
        pack_id,
        object_id,
        object_name,
        material_label,
        recordings,
    } = source
        .adapter_profile
        .as_ref()
        .ok_or_else(|| format!("source {} has no Freesound wine-glass profile", source.id))?
    else {
        return Err(format!(
            "source {} has the wrong Freesound wine-glass profile",
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
            RedistributionPolicy::RedistributableWithNotice
        )
        || pack_id != PACK_ID
        || object_id != OBJECT_ID
        || object_name != OBJECT_NAME
        || material_label != MATERIAL_LABEL
    {
        return Err(format!(
            "source {} does not match the frozen Freesound wine-glass identity",
            source.id
        ));
    }
    if recordings.len() != RECORDINGS.len()
        || recordings.iter().zip(RECORDINGS).any(|(actual, expected)| {
            actual.recording_id != expected.recording_id || actual.sound_id != expected.sound_id
        })
    {
        return Err("Freesound wine-glass recording identities changed".to_owned());
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
            AdapterEvidenceReport::FreesoundWineGlassIdentifiedRecordingV1 {
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
            "source {} must contain three previews and one pack identity",
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
        return Err(
            "Freesound wine-glass pack identity artifact does not match adapter v1".to_owned(),
        );
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
        || pack.author != "wasserbjorn"
        || pack.author_id != AUTHOR_ID
        || pack.pack_id != PACK_ID
        || pack.title != "Metal Clank"
        || pack.description != PACK_DESCRIPTION
        || pack.sounds.len() != 12
    {
        return Err("canonical Freesound wine-glass pack identity changed".to_owned());
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
            || sound.license_label != "Creative Commons 0"
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
                "source {} is missing Freesound wine-glass artifact {id}",
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
        source.artifacts[3].normalization_policy = None;
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
            adapter_profile: Some(AdapterProfile::FreesoundWineGlassIdentifiedRecordingV1 {
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
            redistribution_policy: RedistributionPolicy::RedistributableWithNotice,
            provenance_review: FileRef {
                path: "freesound-wine-glass-provenance-review.md".to_owned(),
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
