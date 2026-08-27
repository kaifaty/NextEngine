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

pub(super) const ADAPTER_ID: &str = "freesound-pack-identified-recording-v1";

const MAXIMUM_PACK_PAGE_BYTES: u64 = 512 * 1024;
const MAXIMUM_PREVIEW_BYTES: u64 = 32 * 1024 * 1024;
const MAXIMUM_RECORDING_SECONDS: u64 = 10 * 60;
const MAX_RECORDINGS: usize = 64;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::physical_sound_registry_command::internet_sources) struct RecordingProfile {
    pub(super) recording_id: String,
    pub(super) sound_id: String,
    pub(super) source_file_name: String,
    pub(super) duration_seconds: String,
    pub(super) sample_rate_hz: u32,
    pub(super) expected_sample_frames: u64,
}

#[derive(Serialize)]
pub(in crate::physical_sound_registry_command::internet_sources) struct RecordingEvidenceReport {
    pub(in crate::physical_sound_registry_command::internet_sources) sound_id: String,
    pub(in crate::physical_sound_registry_command::internet_sources) source_file_name: String,
    pub(in crate::physical_sound_registry_command::internet_sources) preview_variant: &'static str,
    #[serde(flatten)]
    pub(in crate::physical_sound_registry_command::internet_sources) audio: RecordingReport,
}

struct Profile<'a> {
    author: &'a str,
    author_id: &'a str,
    pack_id: &'a str,
    pack_title: &'a str,
    pack_description: &'a str,
    object_id: &'a str,
    object_name: &'a str,
    object_evidence_phrase: &'a str,
    recording_evidence_phrase: &'a str,
    material_label: &'a str,
    license_label: &'a str,
    recordings: &'a [RecordingProfile],
}

pub(super) fn validate_declaration(source: &InternetSource) -> Result<(), String> {
    let profile = profile(source)?;
    validate_profile(&profile)?;

    let landing_page_url = format!(
        "https://freesound.org/people/{}/packs/{}/",
        profile.author, profile.pack_id
    );
    let publisher_id = format!("freesound-user-{}", profile.author);
    let project_id_prefix = format!("pack-{}-", profile.pack_id);
    let source_id_prefix = format!("freesound-pack-{}-", profile.pack_id);
    let object_id_prefix = format!("pack-{}-", profile.pack_id);
    let declared_revision = format!("review-{}", source.review_date);
    let (terms_url, license_expression, redistribution_policy) =
        license_policy(profile.license_label)?;
    if source.publisher_id != publisher_id
        || !source.project_id.starts_with(&project_id_prefix)
        || source.project_id.len() == project_id_prefix.len()
        || !source.id.starts_with(&source_id_prefix)
        || source.id.len() == source_id_prefix.len()
        || !profile.object_id.starts_with(&object_id_prefix)
        || profile.object_id.len() == object_id_prefix.len()
        || source.id != format!("freesound-{}", profile.object_id)
        || source.declared_revision != declared_revision
        || source.landing_page_url != landing_page_url
        || source.terms_url.as_deref() != Some(terms_url)
        || source.license_expression != license_expression
        || source.redistribution_policy.as_str() != redistribution_policy.as_str()
    {
        return Err(format!(
            "source {} does not match its declarative Freesound pack identity",
            source.id
        ));
    }

    validate_artifacts(source, &profile)?;
    validate_capability_evidence(source)?;
    Ok(())
}

pub(super) fn audit(cache: &Path, source: &InternetSource) -> Result<AdapterAudit, String> {
    let profile = profile(source)?;
    let pack_bytes = super::read_cached(cache, artifact(source, "pack-identity")?)?;
    let pack: PackIdentity = serde_json::from_slice(&pack_bytes)
        .map_err(|error| format!("parse canonical Freesound pack identity: {error}"))?;
    validate_pack_identity(source, &profile, &pack)?;

    let mut recordings = Vec::with_capacity(profile.recordings.len());
    for expected in profile.recordings {
        let bytes = super::read_cached(
            cache,
            artifact(source, &format!("impact-{}", expected.recording_id))?,
        )?;
        let audio = mp3::validate_mpeg1_layer3(
            &expected.recording_id,
            &bytes,
            mp3::Expectation {
                source_label: "Freesound HQ preview",
                sample_rate_hz: expected.sample_rate_hz,
                channel_count: 2,
                maximum_sample_frames: expected.expected_sample_frames,
            },
        )?;
        if audio.sample_frames != expected.expected_sample_frames {
            return Err(format!(
                "Freesound sound {} gapless duration changed",
                expected.sound_id
            ));
        }
        recordings.push(RecordingEvidenceReport {
            sound_id: expected.sound_id.clone(),
            source_file_name: expected.source_file_name.clone(),
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
        evidence: Some(AdapterEvidenceReport::FreesoundPackIdentifiedRecordingV1 {
            pack_id: profile.pack_id.to_owned(),
            object_id: profile.object_id.to_owned(),
            object_name: profile.object_name.to_owned(),
            material_label: profile.material_label.to_owned(),
            recordings,
        }),
    })
}

fn profile(source: &InternetSource) -> Result<Profile<'_>, String> {
    let AdapterProfile::FreesoundPackIdentifiedRecordingV1 {
        author,
        author_id,
        pack_id,
        pack_title,
        pack_description,
        object_id,
        object_name,
        object_evidence_phrase,
        recording_evidence_phrase,
        material_label,
        license_label,
        recordings,
    } = source.adapter_profile.as_ref().ok_or_else(|| {
        format!(
            "source {} has no declarative Freesound pack profile",
            source.id
        )
    })?
    else {
        return Err(format!(
            "source {} has the wrong declarative Freesound pack profile",
            source.id
        ));
    };
    Ok(Profile {
        author,
        author_id,
        pack_id,
        pack_title,
        pack_description,
        object_id,
        object_name,
        object_evidence_phrase,
        recording_evidence_phrase,
        material_label,
        license_label,
        recordings,
    })
}

fn validate_profile(profile: &Profile<'_>) -> Result<(), String> {
    validate_identifier(profile.author, 1, 64, "Freesound author")?;
    validate_decimal(profile.author_id, 1, 20, "Freesound author id")?;
    validate_decimal(profile.pack_id, 1, 20, "Freesound pack id")?;
    validate_plain_text(profile.pack_title, 1, 256, "Freesound pack title")?;
    validate_plain_text(
        profile.pack_description,
        0,
        2_048,
        "Freesound pack description",
    )?;
    validate_identifier(profile.object_id, 1, 128, "Freesound object id")?;
    for (value, role) in [
        (profile.object_name, "Freesound object name"),
        (
            profile.object_evidence_phrase,
            "Freesound object evidence phrase",
        ),
        (
            profile.recording_evidence_phrase,
            "Freesound recording evidence phrase",
        ),
    ] {
        validate_plain_text(value, 1, 128, role)?;
    }

    let material_tokens = material_tokens(profile.material_label)?;
    if word_count(profile.object_evidence_phrase) < 2
        || word_count(profile.recording_evidence_phrase) < 2
        || !contains_phrase(profile.object_name, profile.object_evidence_phrase)
        || !contains_any_word(profile.object_evidence_phrase, material_tokens)
        || !contains_any_word(profile.recording_evidence_phrase, material_tokens)
    {
        return Err(
            "Freesound object and recording evidence phrases must identify a material-bearing family"
                .to_owned(),
        );
    }

    let pack_identifies_object =
        contains_phrase(profile.pack_title, profile.object_evidence_phrase)
            || contains_phrase(profile.pack_description, profile.object_evidence_phrase);
    if profile.recordings.len() < 2 || profile.recordings.len() > MAX_RECORDINGS {
        return Err(format!(
            "declarative Freesound recording count must be 2..={MAX_RECORDINGS}"
        ));
    }
    let mut recording_ids = BTreeSet::new();
    let mut sound_ids = BTreeSet::new();
    let mut previous_recording_id: Option<&str> = None;
    for recording in profile.recordings {
        validate_decimal(&recording.recording_id, 3, 3, "Freesound recording id")?;
        validate_decimal(&recording.sound_id, 1, 20, "Freesound sound id")?;
        validate_plain_text(
            &recording.source_file_name,
            1,
            256,
            "Freesound source file name",
        )?;
        validate_decimal_float(&recording.duration_seconds, "Freesound duration")?;
        if !matches!(recording.sample_rate_hz, 32_000 | 44_100 | 48_000)
            || recording.expected_sample_frames == 0
            || recording.expected_sample_frames
                > u64::from(recording.sample_rate_hz) * MAXIMUM_RECORDING_SECONDS
        {
            return Err("Freesound recording format or duration is out of bounds".to_owned());
        }
        if previous_recording_id.is_some_and(|previous| previous >= recording.recording_id.as_str())
            || !recording_ids.insert(recording.recording_id.as_str())
            || !sound_ids.insert(recording.sound_id.as_str())
        {
            return Err(
                "Freesound recording ids must be sorted and sound ids must be unique".to_owned(),
            );
        }
        if !contains_phrase(
            &recording.source_file_name,
            profile.recording_evidence_phrase,
        ) {
            return Err(format!(
                "Freesound sound {} does not identify the declared recording family",
                recording.sound_id
            ));
        }
        previous_recording_id = Some(&recording.recording_id);
    }
    if !pack_identifies_object
        && !profile.recordings.iter().all(|recording| {
            contains_phrase(&recording.source_file_name, profile.object_evidence_phrase)
        })
    {
        return Err(
            "Freesound object evidence phrase is absent from pack and selected sound metadata"
                .to_owned(),
        );
    }
    Ok(())
}

fn validate_artifacts(source: &InternetSource, profile: &Profile<'_>) -> Result<(), String> {
    if source.artifacts.len() != profile.recordings.len() + 1 {
        return Err(format!(
            "source {} must contain one preview per recording and one pack identity",
            source.id
        ));
    }
    for recording in profile.recordings {
        let artifact_id = format!("impact-{}", recording.recording_id);
        let artifact = artifact(source, &artifact_id)?;
        let sound_id = recording
            .sound_id
            .parse::<u64>()
            .map_err(|error| format!("parse Freesound sound id: {error}"))?;
        let preview_url = format!(
            "https://cdn.freesound.org/previews/{}/{}_{}-hq.mp3",
            sound_id / 1_000,
            recording.sound_id,
            profile.author_id
        );
        if artifact.role != ArtifactRole::AudioPayload
            || artifact.url != preview_url
            || artifact.redirect_policy.is_some()
            || artifact.normalization_policy.is_some()
            || artifact.maximum_bytes == 0
            || artifact.maximum_bytes > MAXIMUM_PREVIEW_BYTES
            || artifact.expected_byte_count.is_none()
            || artifact.expected_sha256.is_none()
        {
            return Err(format!(
                "Freesound preview artifact {} does not match the declarative adapter",
                artifact.id
            ));
        }
    }

    let pack = artifact(source, "pack-identity")?;
    if pack.role != ArtifactRole::ProjectDescription
        || pack.url != source.landing_page_url
        || pack.redirect_policy.is_some()
        || pack.normalization_policy != Some(FetchNormalizationPolicy::FreesoundPackIdentityV1)
        || pack.maximum_bytes != MAXIMUM_PACK_PAGE_BYTES
        || pack.expected_byte_count.is_none()
        || pack.expected_sha256.is_none()
    {
        return Err(
            "Freesound pack identity artifact does not match the declarative adapter".to_owned(),
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

fn validate_pack_identity(
    source: &InternetSource,
    profile: &Profile<'_>,
    pack: &PackIdentity,
) -> Result<(), String> {
    if pack.schema != "nextengine.experimental-freesound-pack-identity.v1"
        || pack.source_url != source.landing_page_url
        || pack.author != profile.author
        || pack.author_id != profile.author_id
        || pack.pack_id != profile.pack_id
        || pack.title != profile.pack_title
        || pack.description != profile.pack_description
        || pack.sounds.len() < profile.recordings.len()
        || pack.sounds.len() > MAX_RECORDINGS
    {
        return Err("canonical Freesound pack identity changed".to_owned());
    }
    for expected in profile.recordings {
        let sound = pack
            .sounds
            .iter()
            .find(|sound| sound.sound_id == expected.sound_id)
            .ok_or_else(|| format!("Freesound pack is missing sound {}", expected.sound_id))?;
        let sound_id = expected
            .sound_id
            .parse::<u64>()
            .map_err(|error| format!("parse Freesound sound id: {error}"))?;
        let expected_lq = format!(
            "https://cdn.freesound.org/previews/{}/{}_{}-lq.mp3",
            sound_id / 1_000,
            expected.sound_id,
            profile.author_id
        );
        if sound.title != expected.source_file_name
            || sound.duration_seconds != expected.duration_seconds
            || sound.sample_rate_hz != format!("{}.0", expected.sample_rate_hz)
            || sound.lq_mp3_url != expected_lq
            || sound.license_label != profile.license_label
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
                "source {} is missing declarative Freesound artifact {id}",
                source.id
            )
        })
}

fn license_policy(
    license_label: &str,
) -> Result<(&'static str, &'static str, RedistributionPolicy), String> {
    match license_label {
        "Creative Commons 0" => Ok((
            "https://creativecommons.org/publicdomain/zero/1.0/",
            "CC0-1.0",
            RedistributionPolicy::RedistributableWithNotice,
        )),
        "Attribution" => Ok((
            "https://creativecommons.org/licenses/by/4.0/",
            "CC-BY-4.0",
            RedistributionPolicy::RedistributableWithNotice,
        )),
        "Attribution NonCommercial" => Ok((
            "https://creativecommons.org/licenses/by-nc/4.0/",
            "CC-BY-NC-4.0",
            RedistributionPolicy::ExternalResearchOnly,
        )),
        _ => Err("unsupported Freesound license label".to_owned()),
    }
}

fn material_tokens(material_label: &str) -> Result<&'static [&'static str], String> {
    match material_label {
        "Glass" => Ok(&["glass"]),
        "Ceramic" => Ok(&["ceramic"]),
        "Wood" => Ok(&["wood", "wooden"]),
        "Iron" => Ok(&["iron"]),
        "Steel" => Ok(&["steel"]),
        "Aluminium" => Ok(&["aluminium", "aluminum"]),
        "Plastic" => Ok(&["plastic"]),
        "Polycarbonate" => Ok(&["polycarbonate"]),
        _ => Err("unsupported declarative Freesound material label".to_owned()),
    }
}

fn validate_identifier(
    value: &str,
    minimum: usize,
    maximum: usize,
    role: &str,
) -> Result<(), String> {
    if value.len() < minimum
        || value.len() > maximum
        || !value.is_ascii()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(format!("{role} is not a bounded ASCII identifier"));
    }
    Ok(())
}

fn validate_decimal(value: &str, minimum: usize, maximum: usize, role: &str) -> Result<(), String> {
    if value.len() < minimum
        || value.len() > maximum
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(format!("{role} is not a bounded decimal identifier"));
    }
    Ok(())
}

fn validate_decimal_float(value: &str, role: &str) -> Result<(), String> {
    let mut point = false;
    if value.is_empty()
        || value.len() > 24
        || value.starts_with('.')
        || value.ends_with('.')
        || !value.bytes().all(|byte| {
            if byte == b'.' && !point {
                point = true;
                true
            } else {
                byte.is_ascii_digit()
            }
        })
        || !value.bytes().any(|byte| byte.is_ascii_digit())
    {
        return Err(format!("{role} is not a bounded decimal value"));
    }
    Ok(())
}

fn validate_plain_text(
    value: &str,
    minimum: usize,
    maximum: usize,
    role: &str,
) -> Result<(), String> {
    if value.len() < minimum
        || value.len() > maximum
        || !value.is_ascii()
        || value.bytes().any(|byte| byte.is_ascii_control())
        || value.contains(['<', '>'])
    {
        return Err(format!("{role} is not bounded plain ASCII text"));
    }
    Ok(())
}

fn words(value: &str) -> Vec<String> {
    value
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_ascii_lowercase)
        .collect()
}

fn contains_phrase(haystack: &str, phrase: &str) -> bool {
    let haystack = words(haystack);
    let phrase = words(phrase);
    !phrase.is_empty()
        && haystack
            .windows(phrase.len())
            .any(|window| window == phrase.as_slice())
}

fn contains_any_word(value: &str, expected: &[&str]) -> bool {
    let actual = words(value);
    expected
        .iter()
        .any(|expected| actual.iter().any(|actual| actual == expected))
}

fn word_count(value: &str) -> usize {
    words(value).len()
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
    use crate::physical_sound_registry_command::internet_sources::CapabilityEvidence;

    #[test]
    fn declaration_derives_source_pack_preview_and_license_identity() {
        let source = test_source();
        validate_declaration(&source).expect("valid declarative Freesound source");
    }

    #[test]
    fn declaration_rejects_self_asserted_material_without_publisher_phrase() {
        let mut source = test_source();
        let Some(AdapterProfile::FreesoundPackIdentifiedRecordingV1 {
            object_evidence_phrase,
            recording_evidence_phrase,
            ..
        }) = source.adapter_profile.as_mut()
        else {
            panic!("test profile");
        };
        *object_evidence_phrase = "metal bottle".to_owned();
        *recording_evidence_phrase = "metal bottle".to_owned();
        assert!(validate_declaration(&source).is_err());
    }

    #[test]
    fn declaration_rejects_derived_preview_or_policy_mutation() {
        let mut preview = test_source();
        preview.artifacts[0].url =
            "https://cdn.freesound.org/previews/100/100001_998-hq.mp3".to_owned();
        assert!(validate_declaration(&preview).is_err());

        let mut policy = test_source();
        policy.license_expression = "NOASSERTION".to_owned();
        assert!(validate_declaration(&policy).is_err());
    }

    #[test]
    fn canonical_pack_bytes_must_back_every_selected_recording() {
        let source = test_source();
        let profile = profile(&source).expect("test profile");
        let mut pack = test_pack();
        validate_pack_identity(&source, &profile, &pack).expect("matching pack identity");
        pack.sounds[0].license_label = "Attribution".to_owned();
        assert!(validate_pack_identity(&source, &profile, &pack).is_err());
    }

    fn test_source() -> InternetSource {
        let recordings = vec![
            RecordingProfile {
                recording_id: "001".to_owned(),
                sound_id: "100001".to_owned(),
                source_file_name: "Glass Bottle strike 1".to_owned(),
                duration_seconds: "1.0".to_owned(),
                sample_rate_hz: 44_100,
                expected_sample_frames: 44_100,
            },
            RecordingProfile {
                recording_id: "002".to_owned(),
                sound_id: "100002".to_owned(),
                source_file_name: "Glass Bottle strike 2".to_owned(),
                duration_seconds: "1.0".to_owned(),
                sample_rate_hz: 44_100,
                expected_sample_frames: 44_100,
            },
        ];
        let mut artifacts = recordings
            .iter()
            .map(|recording| RemoteArtifact {
                id: format!("impact-{}", recording.recording_id),
                role: ArtifactRole::AudioPayload,
                url: format!(
                    "https://cdn.freesound.org/previews/100/{}_999-hq.mp3",
                    recording.sound_id
                ),
                redirect_policy: None,
                normalization_policy: None,
                maximum_bytes: 256 * 1024,
                expected_byte_count: Some(1_000),
                expected_sha256: Some("aa".repeat(32)),
            })
            .collect::<Vec<_>>();
        artifacts.push(RemoteArtifact {
            id: "pack-identity".to_owned(),
            role: ArtifactRole::ProjectDescription,
            url: "https://freesound.org/people/tester/packs/123/".to_owned(),
            redirect_policy: None,
            normalization_policy: Some(FetchNormalizationPolicy::FreesoundPackIdentityV1),
            maximum_bytes: MAXIMUM_PACK_PAGE_BYTES,
            expected_byte_count: Some(2_000),
            expected_sha256: Some("bb".repeat(32)),
        });
        let artifact_ids = artifacts
            .iter()
            .map(|artifact| artifact.id.clone())
            .collect::<Vec<_>>();
        InternetSource {
            id: "freesound-pack-123-glass-bottle".to_owned(),
            publisher_id: "freesound-user-tester".to_owned(),
            project_id: "pack-123-glass-bottle-sounds".to_owned(),
            declared_revision: "review-2026-08-28".to_owned(),
            review_date: "2026-08-28".to_owned(),
            landing_page_url: "https://freesound.org/people/tester/packs/123/".to_owned(),
            terms_url: Some("https://creativecommons.org/publicdomain/zero/1.0/".to_owned()),
            adapter_id: ADAPTER_ID.to_owned(),
            adapter_profile: Some(AdapterProfile::FreesoundPackIdentifiedRecordingV1 {
                author: "tester".to_owned(),
                author_id: "999".to_owned(),
                pack_id: "123".to_owned(),
                pack_title: "Glass Bottle sounds".to_owned(),
                pack_description: "One Glass Bottle struck repeatedly.".to_owned(),
                object_id: "pack-123-glass-bottle".to_owned(),
                object_name: "Glass Bottle".to_owned(),
                object_evidence_phrase: "Glass Bottle".to_owned(),
                recording_evidence_phrase: "Glass Bottle".to_owned(),
                material_label: "Glass".to_owned(),
                license_label: "Creative Commons 0".to_owned(),
                recordings,
            }),
            license_expression: "CC0-1.0".to_owned(),
            redistribution_policy: RedistributionPolicy::RedistributableWithNotice,
            provenance_review: FileRef {
                path: "freesound-provenance.md".to_owned(),
                sha256: "cc".repeat(32),
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

    fn test_pack() -> PackIdentity {
        PackIdentity {
            schema: "nextengine.experimental-freesound-pack-identity.v1".to_owned(),
            source_url: "https://freesound.org/people/tester/packs/123/".to_owned(),
            author: "tester".to_owned(),
            author_id: "999".to_owned(),
            pack_id: "123".to_owned(),
            title: "Glass Bottle sounds".to_owned(),
            description: "One Glass Bottle struck repeatedly.".to_owned(),
            sounds: [
                ("100001", "Glass Bottle strike 1"),
                ("100002", "Glass Bottle strike 2"),
            ]
            .into_iter()
            .map(|(sound_id, title)| SoundIdentity {
                sound_id: sound_id.to_owned(),
                title: title.to_owned(),
                duration_seconds: "1.0".to_owned(),
                sample_rate_hz: "44100.0".to_owned(),
                lq_mp3_url: format!("https://cdn.freesound.org/previews/100/{sound_id}_999-lq.mp3"),
                license_label: "Creative Commons 0".to_owned(),
            })
            .collect(),
        }
    }
}
