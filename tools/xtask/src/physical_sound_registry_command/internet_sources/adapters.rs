use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{
    ArtifactRole, CacheStatus, EvidenceCapability, InternetSource, RemoteArtifact,
    cache_artifact_path,
};

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
}

#[derive(Serialize)]
pub(super) struct RecordingReport {
    recording_id: String,
    sample_encoding: &'static str,
    sample_rate_hz: u32,
    channel_count: u16,
    bits_per_sample: u16,
    sample_frames: u64,
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
    if source.adapter_id != AV_MSF_ADAPTER_ID {
        return Ok(AdapterAudit {
            validated_capabilities: BTreeSet::new(),
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
    audit_av_msf(cache, source)
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
        .ok_or_else(|| format!("source {} has no AV-MSF profile", source.id))?;
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
        .ok_or_else(|| format!("source {} has no AV-MSF profile", source.id))?;
    let page = read_cached(cache, artifact(source, "project-page")?)?;
    validate_project_page(&page, object_id, material_label, recording_ids)?;
    let mut recordings = Vec::with_capacity(recording_ids.len());
    for recording_id in recording_ids {
        let bytes = read_cached(cache, artifact(source, &format!("impact-{recording_id}"))?)?;
        recordings.push(validate_float_wav(recording_id, &bytes)?);
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
    let recording_list = recording_ids.join(",");
    for expected in [
        format!("data-original-material=\"{material_label}\""),
        format!("data-demo-path=\"data/demo/{object_id}\""),
        format!("data-contact-impacts=\"{recording_list}\""),
    ] {
        if !card.contains(&expected) {
            return Err(format!(
                "AV-MSF Object {object_id} card is missing {expected}"
            ));
        }
    }
    Ok(())
}

fn validate_float_wav(recording_id: &str, bytes: &[u8]) -> Result<RecordingReport, String> {
    if bytes.len() < 12 || &bytes[..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(format!("AV-MSF impact {recording_id} is not RIFF/WAVE"));
    }
    let declared_size = u32::from_le_bytes(bytes[4..8].try_into().expect("four bytes")) as usize;
    if declared_size.checked_add(8) != Some(bytes.len()) {
        return Err(format!(
            "AV-MSF impact {recording_id} RIFF length does not match the payload"
        ));
    }
    let mut offset = 12_usize;
    let mut format = None;
    let mut fact_frames = None;
    let mut data = None;
    while offset < bytes.len() {
        if bytes.len() - offset < 8 {
            return Err(format!(
                "AV-MSF impact {recording_id} has a truncated chunk"
            ));
        }
        let chunk_id = &bytes[offset..offset + 4];
        let chunk_bytes = u32::from_le_bytes(
            bytes[offset + 4..offset + 8]
                .try_into()
                .expect("four bytes"),
        ) as usize;
        let start = offset + 8;
        let end = start
            .checked_add(chunk_bytes)
            .filter(|end| *end <= bytes.len())
            .ok_or_else(|| format!("AV-MSF impact {recording_id} chunk exceeds the payload"))?;
        match chunk_id {
            b"fmt " => {
                if format.is_some() || chunk_bytes < 16 {
                    return Err(format!(
                        "AV-MSF impact {recording_id} has an invalid fmt chunk"
                    ));
                }
                format = Some(parse_format(recording_id, &bytes[start..end])?);
            }
            b"fact" => {
                if fact_frames.is_some() || chunk_bytes != 4 {
                    return Err(format!(
                        "AV-MSF impact {recording_id} has an invalid fact chunk"
                    ));
                }
                fact_frames = Some(u32::from_le_bytes(
                    bytes[start..end].try_into().expect("four bytes"),
                ) as u64);
            }
            b"data" => {
                if data.is_some() {
                    return Err(format!(
                        "AV-MSF impact {recording_id} has duplicate audio data"
                    ));
                }
                data = Some(&bytes[start..end]);
            }
            _ => {}
        }
        offset = end
            .checked_add(chunk_bytes & 1)
            .filter(|offset| *offset <= bytes.len())
            .ok_or_else(|| format!("AV-MSF impact {recording_id} has invalid chunk padding"))?;
    }
    let format = format.ok_or_else(|| format!("AV-MSF impact {recording_id} has no fmt chunk"))?;
    let data = data.ok_or_else(|| format!("AV-MSF impact {recording_id} has no data chunk"))?;
    if data.is_empty() || data.len() % usize::from(format.block_align) != 0 {
        return Err(format!(
            "AV-MSF impact {recording_id} has misaligned audio data"
        ));
    }
    let sample_frames = (data.len() / usize::from(format.block_align)) as u64;
    if sample_frames > MAX_AV_MSF_RECORDING_FRAMES || fact_frames != Some(sample_frames) {
        return Err(format!(
            "AV-MSF impact {recording_id} has an invalid bounded frame count"
        ));
    }
    let mut any_nonzero = false;
    for sample in data.chunks_exact(4) {
        let value = f32::from_le_bytes(sample.try_into().expect("four bytes"));
        if !value.is_finite() {
            return Err(format!(
                "AV-MSF impact {recording_id} contains a non-finite sample"
            ));
        }
        any_nonzero |= value != 0.0;
    }
    if !any_nonzero {
        return Err(format!("AV-MSF impact {recording_id} is silent"));
    }
    Ok(RecordingReport {
        recording_id: recording_id.to_owned(),
        sample_encoding: "ieee_float32_le",
        sample_rate_hz: format.sample_rate_hz,
        channel_count: format.channel_count,
        bits_per_sample: format.bits_per_sample,
        sample_frames,
    })
}

struct WavFormat {
    sample_rate_hz: u32,
    channel_count: u16,
    bits_per_sample: u16,
    block_align: u16,
}

fn parse_format(recording_id: &str, bytes: &[u8]) -> Result<WavFormat, String> {
    let audio_format = u16::from_le_bytes(bytes[0..2].try_into().expect("two bytes"));
    let channel_count = u16::from_le_bytes(bytes[2..4].try_into().expect("two bytes"));
    let sample_rate_hz = u32::from_le_bytes(bytes[4..8].try_into().expect("four bytes"));
    let byte_rate = u32::from_le_bytes(bytes[8..12].try_into().expect("four bytes"));
    let block_align = u16::from_le_bytes(bytes[12..14].try_into().expect("two bytes"));
    let bits_per_sample = u16::from_le_bytes(bytes[14..16].try_into().expect("two bytes"));
    if audio_format != 3
        || channel_count != 1
        || sample_rate_hz != 44_100
        || bits_per_sample != 32
        || block_align != 4
        || byte_rate != 176_400
    {
        return Err(format!(
            "AV-MSF impact {recording_id} must be mono 44.1 kHz IEEE float32"
        ));
    }
    Ok(WavFormat {
        sample_rate_hz,
        channel_count,
        bits_per_sample,
        block_align,
    })
}
