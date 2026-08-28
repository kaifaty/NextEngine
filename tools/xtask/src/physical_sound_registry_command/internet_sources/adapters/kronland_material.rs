use std::collections::BTreeSet;
use std::path::Path;

use super::{
    AdapterAudit, AdapterEvidenceReport, AdapterProfile, ArtifactRole, EvidenceCapability,
    InternetSource, RemoteArtifact, wav,
};
use crate::physical_sound_registry_command::internet_sources::RedistributionPolicy;

pub(super) const ADAPTER_ID: &str = "kronland-material-identified-recording-v1";

const PUBLISHER_ID: &str = "the-language-of-sounds";
const PROJECT_ID: &str = "kronland-material-impact-stimuli";
const DECLARED_REVISION: &str = "official-public-wav-set-observed-2026-08-28";
const LANDING_PAGE_URL: &str = "https://kronland.fr/publications/controlling-the-perceived-material-in-an-impact-sound-synthesizer/";
const AUDIO_PREFIX: &str = "https://kronland.fr/wp-content/uploads/2007/09/";
const LICENSE_EXPRESSION: &str = "NOASSERTION";
const MATERIAL_LABEL: &str = "Glass";
const PROJECT_PAGE_BYTES: u64 = 235_670;
const PROJECT_PAGE_MAXIMUM_BYTES: u64 = 256 * 1024;
const PROJECT_PAGE_SHA256: &str =
    "d7171401972ff80416bfe8be4d50aaeaf3b5f3164dae9aa790796b8bbaf24beb";
const AUDIO_MAXIMUM_BYTES: u64 = 128 * 1024;
const MAXIMUM_SAMPLE_FRAMES: u64 = 60_000;

#[derive(Clone, Copy)]
struct FrozenObject {
    object_id: &'static str,
    object_name: &'static str,
    recording_id: &'static str,
    source_file_name: &'static str,
    track_number: u8,
    bytes: u64,
    sha256: &'static str,
    sample_frames: u64,
}

const OBJECTS: [FrozenObject; 5] = [
    FrozenObject {
        object_id: "glass-v1",
        object_name: "Glass 1",
        recording_id: "v1",
        source_file_name: "AST_v1_expe.wav",
        track_number: 1,
        bytes: 119_618,
        sha256: "2e950d3a2257a4af2e99407c644fd61280f9f651a200e22d3ca0d8ca7207b400",
        sample_frames: 57_761,
    },
    FrozenObject {
        object_id: "glass-v2",
        object_name: "Glass 2",
        recording_id: "v2",
        source_file_name: "AST_v2_expe.wav",
        track_number: 2,
        bytes: 93_768,
        sha256: "87a649a0be249600bf417c51a6fb410a0201a1fc27a667202e19c2a442adab5d",
        sample_frames: 44_836,
    },
    FrozenObject {
        object_id: "glass-v4",
        object_name: "Glass 3",
        recording_id: "v4",
        source_file_name: "AST_v4_expe.wav",
        track_number: 3,
        bytes: 53_506,
        sha256: "185cb111f532c9760d8c2e956171824a1cb04d48c2ab940fd9e8f5a3bb8031ac",
        sample_frames: 24_705,
    },
    FrozenObject {
        object_id: "glass-v5",
        object_name: "Glass 4",
        recording_id: "v5",
        source_file_name: "AST_v5_expe.wav",
        track_number: 4,
        bytes: 60_832,
        sha256: "869c13ce80568cf2c1b5b6cdb62d5aac58b03470d21f2a146491546e2202b982",
        sample_frames: 28_368,
    },
    FrozenObject {
        object_id: "glass-v6",
        object_name: "Glass 5",
        recording_id: "v6",
        source_file_name: "AST_v6_expe.wav",
        track_number: 5,
        bytes: 96_680,
        sha256: "ec7adb4e2fe6a3b68e13501d993be66addcda285b60eb7d974bbe3fac069ab0a",
        sample_frames: 46_292,
    },
];

pub(super) fn validate_declaration(source: &InternetSource) -> Result<(), String> {
    let (profile, expected) = profile(source)?;
    if source.id != format!("kronland-material-{}", expected.object_id)
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
        || profile.object_name != expected.object_name
        || profile.material_label != MATERIAL_LABEL
        || profile.recording_id != expected.recording_id
        || profile.source_file_name != expected.source_file_name
    {
        return Err(format!(
            "source {} does not match the frozen official Kronland material identity",
            source.id
        ));
    }
    validate_artifacts(source, expected)?;
    validate_capability_evidence(source, expected)?;
    Ok(())
}

pub(super) fn audit(cache: &Path, source: &InternetSource) -> Result<AdapterAudit, String> {
    let (profile, expected) = profile(source)?;
    let page = super::read_cached(cache, artifact(source, "project-page")?)?;
    validate_project_page(&page, expected)?;
    let audio_artifact_id = format!("impact-{}", expected.recording_id);
    let audio_bytes = super::read_cached(cache, artifact(source, &audio_artifact_id)?)?;
    let recording = wav::validate_pcm16_wav(
        expected.recording_id,
        &audio_bytes,
        wav::Pcm16WavExpectation {
            source_label: "Kronland original impact recording",
            sample_rate_hz: 44_100,
            channel_count: 1,
            maximum_frames: MAXIMUM_SAMPLE_FRAMES,
        },
    )?;
    if recording.sample_frames != expected.sample_frames {
        return Err(format!(
            "Kronland recording {} must contain exactly {} frames",
            expected.recording_id, expected.sample_frames
        ));
    }
    Ok(AdapterAudit {
        validated_capabilities: BTreeSet::from([
            EvidenceCapability::MaterialIdentity,
            EvidenceCapability::ObjectIdentity,
            EvidenceCapability::RealRecording,
            EvidenceCapability::RepeatIdentity,
        ]),
        evidence: Some(
            AdapterEvidenceReport::KronlandMaterialIdentifiedRecordingV1 {
                object_id: profile.object_id.to_owned(),
                object_name: profile.object_name.to_owned(),
                material_label: profile.material_label.to_owned(),
                source_file_name: profile.source_file_name.to_owned(),
                recording,
            },
        ),
    })
}

struct Profile<'a> {
    object_id: &'a str,
    object_name: &'a str,
    material_label: &'a str,
    recording_id: &'a str,
    source_file_name: &'a str,
}

fn profile(source: &InternetSource) -> Result<(Profile<'_>, &'static FrozenObject), String> {
    let AdapterProfile::KronlandMaterialIdentifiedRecordingV1 {
        object_id,
        object_name,
        material_label,
        recording_id,
        source_file_name,
    } = source
        .adapter_profile
        .as_ref()
        .ok_or_else(|| format!("source {} has no Kronland material profile", source.id))?
    else {
        return Err(format!(
            "source {} has the wrong Kronland material profile",
            source.id
        ));
    };
    let expected = OBJECTS
        .iter()
        .find(|object| object.object_id == object_id)
        .ok_or_else(|| format!("source {} names an unsupported Kronland object", source.id))?;
    Ok((
        Profile {
            object_id,
            object_name,
            material_label,
            recording_id,
            source_file_name,
        },
        expected,
    ))
}

fn validate_artifacts(source: &InternetSource, expected: &FrozenObject) -> Result<(), String> {
    if source.artifacts.len() != 2 {
        return Err(format!(
            "source {} must contain one Kronland recording and the project page",
            source.id
        ));
    }
    require_artifact(
        artifact(source, &format!("impact-{}", expected.recording_id))?,
        ArtifactRole::AudioPayload,
        &format!("{AUDIO_PREFIX}{}", expected.source_file_name),
        AUDIO_MAXIMUM_BYTES,
        expected.bytes,
        expected.sha256,
    )?;
    require_artifact(
        artifact(source, "project-page")?,
        ArtifactRole::ProjectDescription,
        LANDING_PAGE_URL,
        PROJECT_PAGE_MAXIMUM_BYTES,
        PROJECT_PAGE_BYTES,
        PROJECT_PAGE_SHA256,
    )?;
    Ok(())
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
        || artifact.redirect_policy.is_some()
        || artifact.normalization_policy.is_some()
        || artifact.maximum_bytes != maximum_bytes
        || artifact.expected_byte_count != Some(expected_bytes)
        || artifact.expected_sha256.as_deref() != Some(expected_sha256)
    {
        return Err(format!(
            "Kronland artifact {} does not match adapter v1",
            artifact.id
        ));
    }
    Ok(())
}

fn validate_capability_evidence(
    source: &InternetSource,
    expected: &FrozenObject,
) -> Result<(), String> {
    let artifact_ids = vec![
        format!("impact-{}", expected.recording_id),
        "project-page".to_owned(),
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
            "source {} must bind every E3 capability to its Kronland page and recording",
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
        .ok_or_else(|| format!("source {} is missing Kronland artifact {id}", source.id))
}

fn validate_project_page(bytes: &[u8], expected: &FrozenObject) -> Result<(), String> {
    let page = std::str::from_utf8(bytes)
        .map_err(|error| format!("Kronland project page must be UTF-8: {error}"))?;
    let recorded_scope = "Impact sounds from everyday life objects made of Wood, Metal and Glass materials were recorded";
    let exact_track = format!(
        "<a href='{AUDIO_PREFIX}{}'>Glass {} original</a>",
        expected.source_file_name, expected.track_number
    );
    let synthesized_track = format!(
        "AST_v{}_synth.wav",
        expected.recording_id.trim_start_matches('v')
    );
    let tuned_track = format!(
        "AST_v{}_transp.wav",
        expected.recording_id.trim_start_matches('v')
    );
    if !page.contains("Controlling the perceived material in an impact sound synthesizer")
        || !page.contains("The Language of Sounds")
        || !page.contains(recorded_scope)
        || !page.contains(&exact_track)
        || !page.contains(&synthesized_track)
        || !page.contains(&tuned_track)
    {
        return Err(format!(
            "Kronland page does not bind {} to its original Glass recording",
            expected.object_id
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physical_sound_registry_command::FileRef;
    use crate::physical_sound_registry_command::internet_sources::CapabilityEvidence;

    #[test]
    fn declaration_freezes_each_original_glass_object() {
        for expected in OBJECTS {
            let source = test_source(expected);
            validate_declaration(&source).expect("frozen Kronland declaration");
        }

        let mut source = test_source(OBJECTS[0]);
        let Some(AdapterProfile::KronlandMaterialIdentifiedRecordingV1 { material_label, .. }) =
            source.adapter_profile.as_mut()
        else {
            panic!("Kronland profile");
        };
        *material_label = "Metal".to_owned();
        assert!(validate_declaration(&source).is_err());
    }

    #[test]
    fn page_binding_distinguishes_original_from_derived_tracks() {
        let expected = OBJECTS[0];
        let page = format!(
            "Controlling the perceived material in an impact sound synthesizer — The Language of Sounds\n\
             Impact sounds from everyday life objects made of Wood, Metal and Glass materials were recorded\n\
             <a href='{AUDIO_PREFIX}{}'>Glass 1 original</a>\n\
             AST_v1_synth.wav AST_v1_transp.wav",
            expected.source_file_name
        );
        validate_project_page(page.as_bytes(), &expected).expect("exact original binding");
        assert!(
            validate_project_page(
                page.replace("original", "synthesized").as_bytes(),
                &expected
            )
            .is_err()
        );
    }

    fn test_source(expected: FrozenObject) -> InternetSource {
        let artifact_ids = vec![
            format!("impact-{}", expected.recording_id),
            "project-page".to_owned(),
        ];
        InternetSource {
            id: format!("kronland-material-{}", expected.object_id),
            publisher_id: PUBLISHER_ID.to_owned(),
            project_id: PROJECT_ID.to_owned(),
            declared_revision: DECLARED_REVISION.to_owned(),
            review_date: "2026-08-28".to_owned(),
            landing_page_url: LANDING_PAGE_URL.to_owned(),
            terms_url: Some(LANDING_PAGE_URL.to_owned()),
            adapter_id: ADAPTER_ID.to_owned(),
            adapter_profile: Some(AdapterProfile::KronlandMaterialIdentifiedRecordingV1 {
                object_id: expected.object_id.to_owned(),
                object_name: expected.object_name.to_owned(),
                material_label: MATERIAL_LABEL.to_owned(),
                recording_id: expected.recording_id.to_owned(),
                source_file_name: expected.source_file_name.to_owned(),
            }),
            license_expression: LICENSE_EXPRESSION.to_owned(),
            redistribution_policy: RedistributionPolicy::ExternalResearchOnly,
            provenance_review: FileRef {
                path: "provenance.md".to_owned(),
                sha256: "aa".repeat(32),
            },
            artifacts: vec![
                RemoteArtifact {
                    id: format!("impact-{}", expected.recording_id),
                    role: ArtifactRole::AudioPayload,
                    url: format!("{AUDIO_PREFIX}{}", expected.source_file_name),
                    redirect_policy: None,
                    normalization_policy: None,
                    maximum_bytes: AUDIO_MAXIMUM_BYTES,
                    expected_byte_count: Some(expected.bytes),
                    expected_sha256: Some(expected.sha256.to_owned()),
                },
                RemoteArtifact {
                    id: "project-page".to_owned(),
                    role: ArtifactRole::ProjectDescription,
                    url: LANDING_PAGE_URL.to_owned(),
                    redirect_policy: None,
                    normalization_policy: None,
                    maximum_bytes: PROJECT_PAGE_MAXIMUM_BYTES,
                    expected_byte_count: Some(PROJECT_PAGE_BYTES),
                    expected_sha256: Some(PROJECT_PAGE_SHA256.to_owned()),
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
