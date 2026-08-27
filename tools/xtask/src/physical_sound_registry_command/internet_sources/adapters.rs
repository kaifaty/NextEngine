use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{
    ArtifactRole, CacheStatus, EvidenceCapability, InternetSource, RemoteArtifact,
    cache_artifact_path,
};

pub(super) mod freesound_glass_bowl;
pub(super) mod heller_impact;
mod mp3;
mod wav;
pub(super) mod ycb_impact;
mod zip_archive;

const AV_MSF_ADAPTER_ID: &str = "av-msf-identified-recording-v1";
const AV_MSF_PUBLISHER_ID: &str = "zisen-shao";
const AV_MSF_PROJECT_ID: &str = "av-msf";
const AV_MSF_LANDING_PAGE: &str = "https://zisenshao.github.io/AV-MSF/";
const AV_MSF_RAW_PREFIX: &str = "https://raw.githubusercontent.com/ZisenShao/AV-MSF/";
const MAX_AV_MSF_PROJECT_PAGE_BYTES: u64 = 2 * 1024 * 1024;
const MAX_AV_MSF_RECORDING_BYTES: u64 = 16 * 1024 * 1024;
const MAX_AV_MSF_RECORDING_FRAMES: u64 = 441_000;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, tag = "schema", rename_all = "snake_case")]
pub(super) enum AdapterProfile {
    AvMsfIdentifiedRecordingV1 {
        object_id: String,
        material_label: String,
        recording_ids: Vec<String>,
    },
    HellerImpactIdentifiedRecordingV1 {
        object_id: String,
        object_name: String,
        material_label: String,
        event_label: String,
        recordings: Vec<heller_impact::RecordingProfile>,
    },
    FreesoundGlassBowlIdentifiedRecordingV1 {
        pack_id: String,
        object_id: String,
        object_name: String,
        material_label: String,
        recordings: Vec<freesound_glass_bowl::RecordingProfile>,
    },
    YcbImpactIdentifiedRecordingV1 {
        object_id: String,
        object_name: String,
        primary_material_label: String,
        #[serde(default)]
        secondary_material_label: Option<String>,
        source_split: String,
        recordings: Vec<ycb_impact::RecordingProfile>,
    },
}

pub(super) struct AdapterAudit {
    validated_capabilities: BTreeSet<EvidenceCapability>,
    pub(super) evidence: Option<AdapterEvidenceReport>,
}

impl AdapterAudit {
    pub(super) fn validates(&self, capability: EvidenceCapability) -> bool {
        self.validated_capabilities.contains(&capability)
    }
}

#[derive(Serialize)]
#[serde(tag = "schema", rename_all = "snake_case")]
pub(super) enum AdapterEvidenceReport {
    AvMsfIdentifiedRecordingV1 {
        object_id: String,
        material_label: String,
        recordings: Vec<RecordingReport>,
    },
    HellerImpactIdentifiedRecordingV1 {
        object_id: String,
        object_name: String,
        material_label: String,
        event_label: String,
        recordings: Vec<heller_impact::RecordingEvidenceReport>,
    },
    FreesoundGlassBowlIdentifiedRecordingV1 {
        pack_id: String,
        object_id: String,
        object_name: String,
        material_label: String,
        recordings: Vec<freesound_glass_bowl::RecordingEvidenceReport>,
    },
    YcbImpactIdentifiedRecordingV1 {
        object_id: String,
        object_name: String,
        primary_material_label: String,
        secondary_material_label: Option<String>,
        source_split: String,
        recordings: Vec<ycb_impact::RecordingEvidenceReport>,
    },
}

#[derive(Serialize)]
pub(super) struct RecordingReport {
    pub(super) recording_id: String,
    pub(super) sample_encoding: &'static str,
    pub(super) sample_rate_hz: u32,
    pub(super) channel_count: u16,
    pub(super) bits_per_sample: u16,
    pub(super) sample_frames: u64,
}

pub(super) fn validate_profile_declaration(source: &InternetSource) -> Result<(), String> {
    match (&*source.adapter_id, &source.adapter_profile) {
        ("hash-closed-synthetic-v1", None) => Ok(()),
        (AV_MSF_ADAPTER_ID, Some(AdapterProfile::AvMsfIdentifiedRecordingV1 { .. })) => {
            validate_av_msf_declaration(source)
        }
        (AV_MSF_ADAPTER_ID, None) => Err(format!(
            "source {} requires an AV-MSF adapter profile",
            source.id
        )),
        (
            heller_impact::ADAPTER_ID,
            Some(AdapterProfile::HellerImpactIdentifiedRecordingV1 { .. }),
        ) => heller_impact::validate_declaration(source),
        (heller_impact::ADAPTER_ID, None) => Err(format!(
            "source {} requires a Heller Impact adapter profile",
            source.id
        )),
        (
            freesound_glass_bowl::ADAPTER_ID,
            Some(AdapterProfile::FreesoundGlassBowlIdentifiedRecordingV1 { .. }),
        ) => freesound_glass_bowl::validate_declaration(source),
        (freesound_glass_bowl::ADAPTER_ID, None) => Err(format!(
            "source {} requires a Freesound glass-bowl adapter profile",
            source.id
        )),
        (ycb_impact::ADAPTER_ID, Some(AdapterProfile::YcbImpactIdentifiedRecordingV1 { .. })) => {
            ycb_impact::validate_declaration(source)
        }
        (ycb_impact::ADAPTER_ID, None) => Err(format!(
            "source {} requires a YCB Impact adapter profile",
            source.id
        )),
        (_, Some(_)) => Err(format!(
            "source {} declares a profile for an unsupported adapter",
            source.id
        )),
        (_, None) => Ok(()),
    }
}

pub(super) fn audit(
    cache: &Path,
    source: &InternetSource,
    artifact_statuses: &BTreeMap<String, CacheStatus>,
) -> Result<AdapterAudit, String> {
    if source.adapter_id == "hash-closed-synthetic-v1" {
        return Ok(AdapterAudit {
            validated_capabilities: BTreeSet::from([EvidenceCapability::SyntheticLineage]),
            evidence: None,
        });
    }
    if source
        .artifacts
        .iter()
        .any(|artifact| artifact_statuses.get(&artifact.id) != Some(&CacheStatus::CachedVerified))
    {
        return Ok(AdapterAudit {
            validated_capabilities: BTreeSet::new(),
            evidence: None,
        });
    }
    match source.adapter_id.as_str() {
        AV_MSF_ADAPTER_ID => audit_av_msf(cache, source),
        freesound_glass_bowl::ADAPTER_ID => freesound_glass_bowl::audit(cache, source),
        heller_impact::ADAPTER_ID => heller_impact::audit(cache, source),
        ycb_impact::ADAPTER_ID => ycb_impact::audit(cache, source),
        _ => Ok(AdapterAudit {
            validated_capabilities: BTreeSet::new(),
            evidence: None,
        }),
    }
}

fn validate_av_msf_declaration(source: &InternetSource) -> Result<(), String> {
    if source.publisher_id != AV_MSF_PUBLISHER_ID
        || source.project_id != AV_MSF_PROJECT_ID
        || source.landing_page_url != AV_MSF_LANDING_PAGE
    {
        return Err(format!(
            "source {} does not identify the official AV-MSF project",
            source.id
        ));
    }
    let commit = source
        .declared_revision
        .strip_prefix("commit-")
        .filter(|value| {
            value.len() == 40
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        })
        .ok_or_else(|| {
            format!(
                "source {} AV-MSF revision must be commit-<40 lowercase hex>",
                source.id
            )
        })?;
    let AdapterProfile::AvMsfIdentifiedRecordingV1 {
        object_id,
        material_label,
        recording_ids,
    } = source
        .adapter_profile
        .as_ref()
        .ok_or_else(|| format!("source {} has no AV-MSF profile", source.id))?
    else {
        return Err(format!("source {} has the wrong AV-MSF profile", source.id));
    };
    validate_decimal_id(object_id, 1, 3, "AV-MSF object id")?;
    if material_label.is_empty()
        || material_label.len() > 64
        || !material_label.is_ascii()
        || !material_label
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b' ' | b'-'))
    {
        return Err("AV-MSF material label must be 1..=64 ASCII label bytes".to_owned());
    }
    if recording_ids.is_empty() || recording_ids.len() > 32 {
        return Err("AV-MSF recording count must be 1..=32".to_owned());
    }
    let mut previous: Option<&str> = None;
    for recording_id in recording_ids {
        validate_decimal_id(recording_id, 3, 3, "AV-MSF recording id")?;
        if previous.is_some_and(|value| value >= recording_id.as_str()) {
            return Err("AV-MSF recording ids must be strictly sorted".to_owned());
        }
        previous = Some(recording_id);
    }

    let expected_count = recording_ids.len() + 1;
    if source.artifacts.len() != expected_count {
        return Err(format!(
            "source {} AV-MSF artifact count must be {expected_count}",
            source.id
        ));
    }
    let project_page = artifact(source, "project-page")?;
    require_artifact(
        project_page,
        ArtifactRole::ProjectDescription,
        &format!("{AV_MSF_RAW_PREFIX}{commit}/index.html"),
        MAX_AV_MSF_PROJECT_PAGE_BYTES,
    )?;
    for recording_id in recording_ids {
        let artifact_id = format!("impact-{recording_id}");
        let recording = artifact(source, &artifact_id)?;
        require_artifact(
            recording,
            ArtifactRole::AudioPayload,
            &format!(
                "{AV_MSF_RAW_PREFIX}{commit}/data/demo/{object_id}/contact/impact{recording_id}.wav"
            ),
            MAX_AV_MSF_RECORDING_BYTES,
        )?;
    }
    Ok(())
}

fn validate_decimal_id(
    value: &str,
    minimum: usize,
    maximum: usize,
    role: &str,
) -> Result<(), String> {
    if value.len() < minimum
        || value.len() > maximum
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(format!(
            "{role} must be {minimum}..={maximum} decimal digits"
        ));
    }
    Ok(())
}

fn artifact<'a>(source: &'a InternetSource, id: &str) -> Result<&'a RemoteArtifact, String> {
    source
        .artifacts
        .iter()
        .find(|artifact| artifact.id == id)
        .ok_or_else(|| format!("source {} is missing AV-MSF artifact {id}", source.id))
}

fn require_artifact(
    artifact: &RemoteArtifact,
    role: ArtifactRole,
    url: &str,
    adapter_maximum_bytes: u64,
) -> Result<(), String> {
    if artifact.role != role || artifact.url != url {
        return Err(format!(
            "AV-MSF artifact {} has the wrong role or immutable URL",
            artifact.id
        ));
    }
    let expected_bytes = artifact
        .expected_byte_count
        .ok_or_else(|| format!("AV-MSF artifact {} needs an exact byte count", artifact.id))?;
    if artifact.expected_sha256.is_none()
        || expected_bytes > adapter_maximum_bytes
        || artifact.maximum_bytes > adapter_maximum_bytes
    {
        return Err(format!(
            "AV-MSF artifact {} exceeds its adapter bound or lacks SHA-256",
            artifact.id
        ));
    }
    Ok(())
}

fn audit_av_msf(cache: &Path, source: &InternetSource) -> Result<AdapterAudit, String> {
    let AdapterProfile::AvMsfIdentifiedRecordingV1 {
        object_id,
        material_label,
        recording_ids,
    } = source
        .adapter_profile
        .as_ref()
        .ok_or_else(|| format!("source {} has no AV-MSF profile", source.id))?
    else {
        return Err(format!("source {} has the wrong AV-MSF profile", source.id));
    };
    let page = read_cached(cache, artifact(source, "project-page")?)?;
    validate_project_page(&page, object_id, material_label, recording_ids)?;
    let mut recordings = Vec::with_capacity(recording_ids.len());
    for recording_id in recording_ids {
        let bytes = read_cached(cache, artifact(source, &format!("impact-{recording_id}"))?)?;
        recordings.push(wav::validate_float_wav(
            recording_id,
            &bytes,
            wav::FloatWavExpectation {
                source_label: "AV-MSF impact",
                sample_rate_hz: 44_100,
                channel_count: 1,
                maximum_frames: MAX_AV_MSF_RECORDING_FRAMES,
            },
        )?);
    }
    Ok(AdapterAudit {
        validated_capabilities: BTreeSet::from([
            EvidenceCapability::MaterialIdentity,
            EvidenceCapability::ObjectIdentity,
            EvidenceCapability::RealRecording,
            EvidenceCapability::RepeatIdentity,
        ]),
        evidence: Some(AdapterEvidenceReport::AvMsfIdentifiedRecordingV1 {
            object_id: object_id.clone(),
            material_label: material_label.clone(),
            recordings,
        }),
    })
}

fn read_cached(cache: &Path, artifact: &RemoteArtifact) -> Result<Vec<u8>, String> {
    let sha256 = artifact
        .expected_sha256
        .as_deref()
        .ok_or_else(|| format!("artifact {} has no SHA-256", artifact.id))?;
    let path = cache_artifact_path(cache, sha256)?;
    fs::read(&path).map_err(|error| format!("read adapter artifact {}: {error}", path.display()))
}

fn validate_project_page(
    bytes: &[u8],
    object_id: &str,
    material_label: &str,
    recording_ids: &[String],
) -> Result<(), String> {
    let page = std::str::from_utf8(bytes)
        .map_err(|error| format!("AV-MSF project page must be UTF-8: {error}"))?;
    if !page.contains("Experiments on two real-world") || !page.contains("Impact recordings") {
        return Err("AV-MSF page does not identify real-world impact recordings".to_owned());
    }
    let object_marker = format!("data-name=\"Object {object_id}\"");
    let card_start = page
        .find(&object_marker)
        .ok_or_else(|| format!("AV-MSF page has no Object {object_id} card"))?;
    let card = page[card_start..]
        .split_once("</button>")
        .map(|(card, _)| card)
        .ok_or_else(|| format!("AV-MSF Object {object_id} card is incomplete"))?;
    for expected in [
        format!("data-original-material=\"{material_label}\""),
        format!("data-demo-path=\"data/demo/{object_id}\""),
    ] {
        if !card.contains(&expected) {
            return Err(format!(
                "AV-MSF Object {object_id} card is missing {expected}"
            ));
        }
    }
    validate_recording_attribute(card, object_id, recording_ids)?;
    Ok(())
}

fn validate_recording_attribute(
    card: &str,
    object_id: &str,
    recording_ids: &[String],
) -> Result<(), String> {
    let marker = "data-contact-impacts=\"";
    let values = card
        .split_once(marker)
        .and_then(|(_, remainder)| remainder.split_once('\"'))
        .map(|(values, _)| values)
        .ok_or_else(|| format!("AV-MSF Object {object_id} has no contact-impact identity"))?;
    let mut page_ids = values.split(',').map(str::to_owned).collect::<Vec<_>>();
    for recording_id in &page_ids {
        validate_decimal_id(recording_id, 3, 3, "AV-MSF page recording id")?;
    }
    page_ids.sort();
    if page_ids != recording_ids {
        return Err(format!(
            "AV-MSF Object {object_id} contact-impact identities do not match the profile"
        ));
    }
    Ok(())
}
