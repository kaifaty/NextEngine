use std::collections::BTreeSet;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

use super::{
    AdapterAudit, AdapterEvidenceReport, AdapterProfile, ArtifactRole, EvidenceCapability,
    InternetSource, RecordingReport, RemoteArtifact, mp4,
};
use crate::physical_sound_registry_command::internet_sources::RedistributionPolicy;

pub(super) const ADAPTER_ID: &str = "objectfolder-real-demo-identified-recording-v1";

const PUBLISHER_ID: &str = "stanford-vision-and-learning-lab";
const PROJECT_ID: &str = "objectfolder-real-interactive-demos";
const DECLARED_REVISION: &str = "demo-repositories-2023-02-09-v1";
const LANDING_PAGE_URL: &str = "https://objectfolder.stanford.edu/objectfolder-real-download";
const DEMO_INDEX_URL: &str = "https://objectfolder.stanford.edu/objectfolder-real";
const MAXIMUM_OFFICIAL_PAGE_BYTES: u64 = 512 * 1024;
const MAXIMUM_GITHUB_JSON_BYTES: u64 = 4 * 1024 * 1024;
const MAXIMUM_RECORDING_BYTES: u64 = 2 * 1024 * 1024;
const SAMPLE_RATE_HZ: u32 = 44_100;
const PLAYABLE_SAMPLE_FRAMES: u64 = 264_997;
const MAXIMUM_SAMPLE_FRAMES: u64 = SAMPLE_RATE_HZ as u64 * 10;
const MINIMUM_RECORDINGS: usize = 3;
const MAXIMUM_RECORDINGS: usize = 16;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::physical_sound_registry_command::internet_sources) struct RecordingProfile {
    pub(super) recording_id: String,
    pub(super) clip_index: u16,
    pub(super) git_blob_sha1: String,
}

#[derive(Serialize)]
pub(in crate::physical_sound_registry_command::internet_sources) struct RecordingEvidenceReport {
    pub(in crate::physical_sound_registry_command::internet_sources) clip_index: u16,
    pub(in crate::physical_sound_registry_command::internet_sources) git_blob_sha1: String,
    pub(in crate::physical_sound_registry_command::internet_sources) source_file_name: &'static str,
    pub(in crate::physical_sound_registry_command::internet_sources) container_format: &'static str,
    #[serde(flatten)]
    pub(in crate::physical_sound_registry_command::internet_sources) audio: RecordingReport,
}

struct Profile<'a> {
    object_id: &'a str,
    object_name: &'a str,
    material_label: &'a str,
    repository_name: &'a str,
    repository_commit: &'a str,
    repository_tree_sha1: &'a str,
    recordings: &'a [RecordingProfile],
}

#[derive(Deserialize)]
struct GithubCommit {
    sha: String,
    commit: GithubCommitBody,
}

#[derive(Deserialize)]
struct GithubCommitBody {
    tree: GithubTreeIdentity,
}

#[derive(Deserialize)]
struct GithubTreeIdentity {
    sha: String,
}

#[derive(Deserialize)]
struct GithubTree {
    sha: String,
    truncated: bool,
}

pub(super) fn validate_declaration(source: &InternetSource) -> Result<(), String> {
    let profile = profile(source)?;
    validate_profile(&profile)?;
    if source.id != format!("objectfolder-real-demo-{}", profile.object_id)
        || source.publisher_id != PUBLISHER_ID
        || source.project_id != PROJECT_ID
        || source.declared_revision != DECLARED_REVISION
        || source.landing_page_url != LANDING_PAGE_URL
        || source.terms_url.is_some()
        || source.license_expression != "NOASSERTION"
        || !matches!(
            source.redistribution_policy,
            RedistributionPolicy::ExternalResearchOnly
        )
    {
        return Err(format!(
            "source {} does not match the ObjectFolder-Real demo identity",
            source.id
        ));
    }
    validate_artifacts(source, &profile)?;
    validate_capability_evidence(source)?;
    Ok(())
}

pub(super) fn audit(cache: &Path, source: &InternetSource) -> Result<AdapterAudit, String> {
    let profile = profile(source)?;
    let object_table = super::read_cached(cache, artifact(source, "official-object-table")?)?;
    let demo_index = super::read_cached(cache, artifact(source, "official-demo-index")?)?;
    validate_official_identity(&profile, &object_table, &demo_index)?;

    let commit_bytes = super::read_cached(cache, artifact(source, "repository-commit")?)?;
    let commit: GithubCommit = serde_json::from_slice(&commit_bytes)
        .map_err(|error| format!("parse ObjectFolder demo GitHub commit: {error}"))?;
    if commit.sha != profile.repository_commit
        || commit.commit.tree.sha != profile.repository_tree_sha1
    {
        return Err("ObjectFolder demo repository commit identity changed".to_owned());
    }

    let tree_bytes = super::read_cached(cache, artifact(source, "repository-tree")?)?;
    let tree: GithubTree = serde_json::from_slice(&tree_bytes)
        .map_err(|error| format!("parse ObjectFolder demo GitHub tree: {error}"))?;
    if tree.sha != profile.repository_tree_sha1 || tree.truncated {
        return Err("ObjectFolder demo repository tree identity changed".to_owned());
    }

    let mut recordings = Vec::with_capacity(profile.recordings.len());
    for expected in profile.recordings {
        let artifact_id = format!("impact-{}", expected.recording_id);
        let recording_artifact = artifact(source, &artifact_id)?;
        let bytes = super::read_cached(cache, recording_artifact)?;
        if git_blob_sha1(&bytes) != expected.git_blob_sha1 {
            return Err(format!(
                "ObjectFolder demo Git blob identity changed for recording {}",
                expected.recording_id
            ));
        }
        let audio = mp4::validate_mpeg_layer3_isobmff(
            &expected.recording_id,
            &bytes,
            mp4::Expectation {
                source_label: "ObjectFolder-Real interactive recording",
                sample_rate_hz: SAMPLE_RATE_HZ,
                channel_count: 2,
                expected_sample_frames: PLAYABLE_SAMPLE_FRAMES,
                maximum_sample_frames: MAXIMUM_SAMPLE_FRAMES,
            },
        )?;
        recordings.push(RecordingEvidenceReport {
            clip_index: expected.clip_index,
            git_blob_sha1: expected.git_blob_sha1.clone(),
            source_file_name: "audio_vid.mp4",
            container_format: "iso_base_media",
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
            AdapterEvidenceReport::ObjectfolderRealDemoIdentifiedRecordingV1 {
                object_id: profile.object_id.to_owned(),
                object_name: profile.object_name.to_owned(),
                material_label: profile.material_label.to_owned(),
                repository_name: profile.repository_name.to_owned(),
                repository_commit: profile.repository_commit.to_owned(),
                recordings,
            },
        ),
    })
}

fn profile(source: &InternetSource) -> Result<Profile<'_>, String> {
    let AdapterProfile::ObjectfolderRealDemoIdentifiedRecordingV1 {
        object_id,
        object_name,
        material_label,
        repository_name,
        repository_commit,
        repository_tree_sha1,
        recordings,
    } = source
        .adapter_profile
        .as_ref()
        .ok_or_else(|| format!("source {} has no ObjectFolder-Real demo profile", source.id))?
    else {
        return Err(format!(
            "source {} has the wrong ObjectFolder-Real demo profile",
            source.id
        ));
    };
    Ok(Profile {
        object_id,
        object_name,
        material_label,
        repository_name,
        repository_commit,
        repository_tree_sha1,
        recordings,
    })
}

fn validate_profile(profile: &Profile<'_>) -> Result<(), String> {
    let object_id = profile
        .object_id
        .parse::<u16>()
        .map_err(|error| format!("parse ObjectFolder object id: {error}"))?;
    if !(1..=100).contains(&object_id)
        || !is_identifier(profile.object_name, 1, 64, true)
        || !matches!(
            profile.material_label,
            "Ceramic" | "Glass" | "Wood" | "Plastic" | "Iron" | "Polycarbonate" | "Steel"
        )
        || !is_identifier(profile.repository_name, 1, 64, true)
        || !profile.repository_name.ends_with("_vis")
        || !is_sha1(profile.repository_commit)
        || !is_sha1(profile.repository_tree_sha1)
        || profile.recordings.len() < MINIMUM_RECORDINGS
        || profile.recordings.len() > MAXIMUM_RECORDINGS
    {
        return Err("ObjectFolder-Real demo profile is out of bounds".to_owned());
    }
    let mut previous: Option<&str> = None;
    let mut clip_indices = BTreeSet::new();
    let mut blob_ids = BTreeSet::new();
    for recording in profile.recordings {
        if recording.recording_id != format!("{:03}", recording.clip_index)
            || previous.is_some_and(|value| value >= recording.recording_id.as_str())
            || !clip_indices.insert(recording.clip_index)
            || !is_sha1(&recording.git_blob_sha1)
            || !blob_ids.insert(recording.git_blob_sha1.as_str())
        {
            return Err(
                "ObjectFolder-Real recording ids must be sorted, exact and unique".to_owned(),
            );
        }
        previous = Some(&recording.recording_id);
    }
    Ok(())
}

fn validate_artifacts(source: &InternetSource, profile: &Profile<'_>) -> Result<(), String> {
    if source.artifacts.len() != profile.recordings.len() + 4 {
        return Err(format!(
            "source {} must contain recordings plus four identity artifacts",
            source.id
        ));
    }
    for recording in profile.recordings {
        let artifact_id = format!("impact-{}", recording.recording_id);
        let recording_artifact = artifact(source, &artifact_id)?;
        let expected_url = format!(
            "https://raw.githubusercontent.com/objectfolder/{}/{}/{}",
            profile.repository_name,
            profile.repository_commit,
            recording_path(profile.object_id, recording.clip_index)
        );
        if recording_artifact.role != ArtifactRole::AudioPayload
            || recording_artifact.url != expected_url
            || recording_artifact.redirect_policy.is_some()
            || recording_artifact.normalization_policy.is_some()
            || recording_artifact.maximum_bytes != MAXIMUM_RECORDING_BYTES
            || recording_artifact.expected_byte_count.is_none()
            || recording_artifact.expected_sha256.is_none()
        {
            return Err(format!(
                "ObjectFolder-Real recording artifact {artifact_id} is not exact"
            ));
        }
    }
    validate_metadata_artifact(
        artifact(source, "official-demo-index")?,
        ArtifactRole::ProjectDescription,
        DEMO_INDEX_URL,
        MAXIMUM_OFFICIAL_PAGE_BYTES,
    )?;
    validate_metadata_artifact(
        artifact(source, "official-object-table")?,
        ArtifactRole::Metadata,
        LANDING_PAGE_URL,
        MAXIMUM_OFFICIAL_PAGE_BYTES,
    )?;
    validate_metadata_artifact(
        artifact(source, "repository-commit")?,
        ArtifactRole::Metadata,
        &format!(
            "https://api.github.com/repos/objectfolder/{}/commits/{}",
            profile.repository_name, profile.repository_commit
        ),
        MAXIMUM_GITHUB_JSON_BYTES,
    )?;
    validate_metadata_artifact(
        artifact(source, "repository-tree")?,
        ArtifactRole::Metadata,
        &format!(
            "https://api.github.com/repos/objectfolder/{}/git/trees/{}",
            profile.repository_name, profile.repository_tree_sha1
        ),
        MAXIMUM_GITHUB_JSON_BYTES,
    )?;
    Ok(())
}

fn validate_metadata_artifact(
    artifact: &RemoteArtifact,
    role: ArtifactRole,
    url: &str,
    maximum_bytes: u64,
) -> Result<(), String> {
    if artifact.role != role
        || artifact.url != url
        || artifact.redirect_policy.is_some()
        || artifact.normalization_policy.is_some()
        || artifact.maximum_bytes != maximum_bytes
        || artifact.expected_byte_count.is_none()
        || artifact.expected_sha256.is_none()
    {
        return Err(format!(
            "ObjectFolder-Real metadata artifact {} is not exact",
            artifact.id
        ));
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
            "source {} must bind each E3 capability to every ObjectFolder artifact",
            source.id
        ));
    }
    Ok(())
}

fn validate_official_identity(
    profile: &Profile<'_>,
    object_table: &[u8],
    demo_index: &[u8],
) -> Result<(), String> {
    let object_table = std::str::from_utf8(object_table)
        .map_err(|error| format!("decode ObjectFolder object table: {error}"))?;
    let demo_index = std::str::from_utf8(demo_index)
        .map_err(|error| format!("decode ObjectFolder demo index: {error}"))?;
    let table_row = format!(
        "<td style=\"text-align: center\">{}</td>\n      <td style=\"text-align: center\">{}</td>\n      <td style=\"text-align: center\">{}</td>",
        profile.object_id, profile.object_name, profile.material_label
    );
    let demo_binding = format!(
        "<a href=\"https://www.objectfolder.org/{}/\">\n        <img src=\"/assets/img/objectfolder/samples/{}/visual/0.png\"",
        profile.repository_name, profile.object_id
    );
    if !object_table.contains("100 real-world household objects")
        || !object_table.contains("impact sound recordings recorded at 30–50 points")
        || !object_table.contains(&table_row)
        || !demo_index.contains("interactive demos for six representative objects")
        || !demo_index.contains(&demo_binding)
    {
        return Err(format!(
            "official ObjectFolder metadata does not back object {}",
            profile.object_id
        ));
    }
    Ok(())
}

fn recording_path(object_id: &str, clip_index: u16) -> String {
    format!("StreamingAssets/{object_id}/{clip_index}/audio_vid.mp4")
}

fn git_blob_sha1(bytes: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(format!("blob {}\0", bytes.len()).as_bytes());
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn artifact<'a>(source: &'a InternetSource, id: &str) -> Result<&'a RemoteArtifact, String> {
    source
        .artifacts
        .iter()
        .find(|artifact| artifact.id == id)
        .ok_or_else(|| format!("source {} is missing artifact {id}", source.id))
}

fn is_identifier(value: &str, minimum: usize, maximum: usize, allow_uppercase: bool) -> bool {
    value.len() >= minimum
        && value.len() <= maximum
        && value.bytes().all(|byte| {
            byte.is_ascii_digit()
                || byte == b'_'
                || byte == b'-'
                || byte.is_ascii_lowercase()
                || (allow_uppercase && byte.is_ascii_uppercase())
        })
}

fn is_sha1(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::{
        AdapterProfile, DECLARED_REVISION, DEMO_INDEX_URL, LANDING_PAGE_URL,
        MAXIMUM_GITHUB_JSON_BYTES, MAXIMUM_OFFICIAL_PAGE_BYTES, MAXIMUM_RECORDING_BYTES,
        RecordingProfile, validate_declaration,
    };
    use crate::physical_sound_registry_command::FileRef;
    use crate::physical_sound_registry_command::internet_sources::{
        ArtifactRole, CapabilityEvidence, EvidenceCapability, InternetSource, RedistributionPolicy,
        RemoteArtifact,
    };

    #[test]
    fn declaration_derives_all_remote_identities() {
        validate_declaration(&source()).expect("declaration must validate");
    }

    #[test]
    fn git_blob_identity_includes_header_and_payload() {
        assert_eq!(
            super::git_blob_sha1(b"test\n"),
            "9daeafb9864cf43055ae93beb0afd6c7d144bfa4"
        );
    }

    #[test]
    fn declaration_rejects_arbitrary_repository_or_recording_url() {
        let mut wrong_repository = source();
        let AdapterProfile::ObjectfolderRealDemoIdentifiedRecordingV1 {
            repository_name, ..
        } = wrong_repository.adapter_profile.as_mut().unwrap()
        else {
            unreachable!();
        };
        *repository_name = "other_vis".to_owned();
        assert!(validate_declaration(&wrong_repository).is_err());

        let mut wrong_recording = source();
        wrong_recording.artifacts[0].url =
            "https://raw.githubusercontent.com/objectfolder/bowl_vis/main/arbitrary.mp4".to_owned();
        assert!(validate_declaration(&wrong_recording).is_err());
    }

    fn source() -> InternetSource {
        let recordings = vec![
            RecordingProfile {
                recording_id: "000".to_owned(),
                clip_index: 0,
                git_blob_sha1: "1111111111111111111111111111111111111111".to_owned(),
            },
            RecordingProfile {
                recording_id: "020".to_owned(),
                clip_index: 20,
                git_blob_sha1: "2222222222222222222222222222222222222222".to_owned(),
            },
            RecordingProfile {
                recording_id: "039".to_owned(),
                clip_index: 39,
                git_blob_sha1: "3333333333333333333333333333333333333333".to_owned(),
            },
        ];
        let commit = "4444444444444444444444444444444444444444";
        let tree = "5555555555555555555555555555555555555555";
        let mut artifacts = recordings
            .iter()
            .map(|recording| RemoteArtifact {
                id: format!("impact-{}", recording.recording_id),
                role: ArtifactRole::AudioPayload,
                url: format!(
                    "https://raw.githubusercontent.com/objectfolder/bowl_vis/{commit}/StreamingAssets/6/{}/audio_vid.mp4",
                    recording.clip_index
                ),
                redirect_policy: None,
                normalization_policy: None,
                maximum_bytes: MAXIMUM_RECORDING_BYTES,
                expected_byte_count: Some(1),
                expected_sha256: Some("a".repeat(64)),
            })
            .collect::<Vec<_>>();
        artifacts.extend([
            metadata(
                "official-demo-index",
                ArtifactRole::ProjectDescription,
                DEMO_INDEX_URL,
                MAXIMUM_OFFICIAL_PAGE_BYTES,
            ),
            metadata(
                "official-object-table",
                ArtifactRole::Metadata,
                LANDING_PAGE_URL,
                MAXIMUM_OFFICIAL_PAGE_BYTES,
            ),
            metadata(
                "repository-commit",
                ArtifactRole::Metadata,
                &format!("https://api.github.com/repos/objectfolder/bowl_vis/commits/{commit}"),
                MAXIMUM_GITHUB_JSON_BYTES,
            ),
            metadata(
                "repository-tree",
                ArtifactRole::Metadata,
                &format!("https://api.github.com/repos/objectfolder/bowl_vis/git/trees/{tree}"),
                MAXIMUM_GITHUB_JSON_BYTES,
            ),
        ]);
        let artifact_ids = artifacts
            .iter()
            .map(|artifact| artifact.id.clone())
            .collect::<Vec<_>>();
        InternetSource {
            id: "objectfolder-real-demo-6".to_owned(),
            publisher_id: "stanford-vision-and-learning-lab".to_owned(),
            project_id: "objectfolder-real-interactive-demos".to_owned(),
            declared_revision: DECLARED_REVISION.to_owned(),
            review_date: "2026-08-28".to_owned(),
            landing_page_url: LANDING_PAGE_URL.to_owned(),
            terms_url: None,
            adapter_id: super::ADAPTER_ID.to_owned(),
            adapter_profile: Some(AdapterProfile::ObjectfolderRealDemoIdentifiedRecordingV1 {
                object_id: "6".to_owned(),
                object_name: "Blue_Bowl".to_owned(),
                material_label: "Glass".to_owned(),
                repository_name: "bowl_vis".to_owned(),
                repository_commit: commit.to_owned(),
                repository_tree_sha1: tree.to_owned(),
                recordings,
            }),
            license_expression: "NOASSERTION".to_owned(),
            redistribution_policy: RedistributionPolicy::ExternalResearchOnly,
            provenance_review: FileRef {
                path: "provenance-review.md".to_owned(),
                sha256: "b".repeat(64),
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

    fn metadata(id: &str, role: ArtifactRole, url: &str, maximum_bytes: u64) -> RemoteArtifact {
        RemoteArtifact {
            id: id.to_owned(),
            role,
            url: url.to_owned(),
            redirect_policy: None,
            normalization_policy: None,
            maximum_bytes,
            expected_byte_count: Some(1),
            expected_sha256: Some("c".repeat(64)),
        }
    }
}
