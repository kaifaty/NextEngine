use std::collections::BTreeSet;
use std::path::Path;

use serde::Deserialize;

use super::{
    AdapterAudit, AdapterEvidenceReport, AdapterProfile, ArtifactRole, EvidenceCapability,
    FetchNormalizationPolicy, InternetSource, RemoteArtifact, wav,
};
use crate::physical_sound_registry_command::internet_sources::RedistributionPolicy;

pub(super) const ADAPTER_ID: &str = "kronland-material-identified-recording-v1";

const PUBLISHER_ID: &str = "the-language-of-sounds";
const PROJECT_ID: &str = "kronland-material-impact-stimuli";
const DECLARED_REVISION: &str = "official-public-wav-set-observed-2026-08-28";
const LANDING_PAGE_URL: &str = "https://kronland.fr/publications/controlling-the-perceived-material-in-an-impact-sound-synthesizer/";
const AUDIO_PREFIX: &str = "https://kronland.fr/wp-content/uploads/2007/09/";
const LICENSE_EXPRESSION: &str = "NOASSERTION";
const PROJECT_PAGE_BYTES: u64 = 235_670;
const PROJECT_PAGE_MAXIMUM_BYTES: u64 = 256 * 1024;
const PROJECT_PAGE_SHA256: &str =
    "d7171401972ff80416bfe8be4d50aaeaf3b5f3164dae9aa790796b8bbaf24beb";
const NORMALIZED_PROJECT_PAGE_SCHEMA: &str =
    "nextengine.experimental-kronland-material-page-identity.v1";
const NORMALIZED_PROJECT_PAGE_BYTES: u64 = 5_127;
const NORMALIZED_PROJECT_PAGE_SHA256: &str =
    "2d01dadce8fbb43b2cc1ed7cabf1f3531873afc527a991990ca1b8a81c588c34";
const SHORT_AUDIO_MAXIMUM_BYTES: u64 = 128 * 1024;
const LONG_AUDIO_MAXIMUM_BYTES: u64 = 1024 * 1024;
const SHORT_AUDIO_MAXIMUM_FRAMES: u64 = 60_000;
const LONG_AUDIO_MAXIMUM_FRAMES: u64 = 400_000;

#[derive(Clone, Copy)]
struct FrozenObject {
    object_id: &'static str,
    object_name: &'static str,
    material_label: &'static str,
    recording_id: &'static str,
    source_file_name: &'static str,
    track_number: u8,
    maximum_bytes: u64,
    bytes: u64,
    sha256: &'static str,
    maximum_sample_frames: u64,
    sample_frames: u64,
}

const OBJECTS: [FrozenObject; 15] = [
    FrozenObject {
        object_id: "glass-v1",
        object_name: "Glass 1",
        material_label: "Glass",
        recording_id: "v1",
        source_file_name: "AST_v1_expe.wav",
        track_number: 1,
        maximum_bytes: SHORT_AUDIO_MAXIMUM_BYTES,
        bytes: 119_618,
        sha256: "2e950d3a2257a4af2e99407c644fd61280f9f651a200e22d3ca0d8ca7207b400",
        maximum_sample_frames: SHORT_AUDIO_MAXIMUM_FRAMES,
        sample_frames: 57_761,
    },
    FrozenObject {
        object_id: "glass-v2",
        object_name: "Glass 2",
        material_label: "Glass",
        recording_id: "v2",
        source_file_name: "AST_v2_expe.wav",
        track_number: 2,
        maximum_bytes: SHORT_AUDIO_MAXIMUM_BYTES,
        bytes: 93_768,
        sha256: "87a649a0be249600bf417c51a6fb410a0201a1fc27a667202e19c2a442adab5d",
        maximum_sample_frames: SHORT_AUDIO_MAXIMUM_FRAMES,
        sample_frames: 44_836,
    },
    FrozenObject {
        object_id: "glass-v4",
        object_name: "Glass 3",
        material_label: "Glass",
        recording_id: "v4",
        source_file_name: "AST_v4_expe.wav",
        track_number: 3,
        maximum_bytes: SHORT_AUDIO_MAXIMUM_BYTES,
        bytes: 53_506,
        sha256: "185cb111f532c9760d8c2e956171824a1cb04d48c2ab940fd9e8f5a3bb8031ac",
        maximum_sample_frames: SHORT_AUDIO_MAXIMUM_FRAMES,
        sample_frames: 24_705,
    },
    FrozenObject {
        object_id: "glass-v5",
        object_name: "Glass 4",
        material_label: "Glass",
        recording_id: "v5",
        source_file_name: "AST_v5_expe.wav",
        track_number: 4,
        maximum_bytes: SHORT_AUDIO_MAXIMUM_BYTES,
        bytes: 60_832,
        sha256: "869c13ce80568cf2c1b5b6cdb62d5aac58b03470d21f2a146491546e2202b982",
        maximum_sample_frames: SHORT_AUDIO_MAXIMUM_FRAMES,
        sample_frames: 28_368,
    },
    FrozenObject {
        object_id: "glass-v6",
        object_name: "Glass 5",
        material_label: "Glass",
        recording_id: "v6",
        source_file_name: "AST_v6_expe.wav",
        track_number: 5,
        maximum_bytes: SHORT_AUDIO_MAXIMUM_BYTES,
        bytes: 96_680,
        sha256: "ec7adb4e2fe6a3b68e13501d993be66addcda285b60eb7d974bbe3fac069ab0a",
        maximum_sample_frames: SHORT_AUDIO_MAXIMUM_FRAMES,
        sample_frames: 46_292,
    },
    FrozenObject {
        object_id: "wood-b1",
        object_name: "Wood 1",
        material_label: "Wood",
        recording_id: "b1",
        source_file_name: "AST_b1_expe.wav",
        track_number: 1,
        maximum_bytes: SHORT_AUDIO_MAXIMUM_BYTES,
        bytes: 59_114,
        sha256: "355d333c1c04dec5151e7f6b1a50b0ec8f2751ff6903a47b51e5d200205f02a8",
        maximum_sample_frames: SHORT_AUDIO_MAXIMUM_FRAMES,
        sample_frames: 27_509,
    },
    FrozenObject {
        object_id: "wood-b2",
        object_name: "Wood 2",
        material_label: "Wood",
        recording_id: "b2",
        source_file_name: "AST_b2_expe.wav",
        track_number: 2,
        maximum_bytes: SHORT_AUDIO_MAXIMUM_BYTES,
        bytes: 113_264,
        sha256: "7c0f5f083b22fc7f577dba40a713be5f9d5865b2fd5a23237f895e556d60adca",
        maximum_sample_frames: SHORT_AUDIO_MAXIMUM_FRAMES,
        sample_frames: 54_584,
    },
    FrozenObject {
        object_id: "wood-b3",
        object_name: "Wood 3",
        material_label: "Wood",
        recording_id: "b3",
        source_file_name: "AST_b3_expe.wav",
        track_number: 3,
        maximum_bytes: SHORT_AUDIO_MAXIMUM_BYTES,
        bytes: 122_350,
        sha256: "5f7ceb98564071d363d0aafc08d047c82fddcd38d3838b30a30dc93f16ec7270",
        maximum_sample_frames: SHORT_AUDIO_MAXIMUM_FRAMES,
        sample_frames: 59_127,
    },
    FrozenObject {
        object_id: "wood-b4",
        object_name: "Wood 4",
        material_label: "Wood",
        recording_id: "b4",
        source_file_name: "AST_b4_expe.wav",
        track_number: 4,
        maximum_bytes: SHORT_AUDIO_MAXIMUM_BYTES,
        bytes: 119_752,
        sha256: "eb488cff78ca90e7453014dbf5caa4aa1d1c0cf2c931b3e1c17f84add84fde20",
        maximum_sample_frames: SHORT_AUDIO_MAXIMUM_FRAMES,
        sample_frames: 57_828,
    },
    FrozenObject {
        object_id: "wood-b5",
        object_name: "Wood 5",
        material_label: "Wood",
        recording_id: "b5",
        source_file_name: "AST_b5_expe.wav",
        track_number: 5,
        maximum_bytes: SHORT_AUDIO_MAXIMUM_BYTES,
        bytes: 95_570,
        sha256: "9dbcb8b6ea8d03d2e8110b2e959bf5c2ab4bf376fab2b1389b30644a857e52b3",
        maximum_sample_frames: SHORT_AUDIO_MAXIMUM_FRAMES,
        sample_frames: 45_737,
    },
    FrozenObject {
        object_id: "metal-m1",
        object_name: "Metal 1",
        material_label: "Metal",
        recording_id: "m1",
        source_file_name: "AST_m1_expe.wav",
        track_number: 1,
        maximum_bytes: LONG_AUDIO_MAXIMUM_BYTES,
        bytes: 274_662,
        sha256: "a1dad3b297bdfaa1a5e3e18b8643184875bf2efc9af9ad9ed75a72919b7de7fa",
        maximum_sample_frames: LONG_AUDIO_MAXIMUM_FRAMES,
        sample_frames: 135_283,
    },
    FrozenObject {
        object_id: "metal-m5",
        object_name: "Metal 2",
        material_label: "Metal",
        recording_id: "m5",
        source_file_name: "AST_m5_expe.wav",
        track_number: 2,
        maximum_bytes: LONG_AUDIO_MAXIMUM_BYTES,
        bytes: 615_120,
        sha256: "993f858d0237c42ac89ee032ac25ff452fd920aedd0a505bcddcf491c0d3ae31",
        maximum_sample_frames: LONG_AUDIO_MAXIMUM_FRAMES,
        sample_frames: 305_512,
    },
    FrozenObject {
        object_id: "metal-m8",
        object_name: "Metal 3",
        material_label: "Metal",
        recording_id: "m8",
        source_file_name: "AST_m8_expe.wav",
        track_number: 3,
        maximum_bytes: LONG_AUDIO_MAXIMUM_BYTES,
        bytes: 794_862,
        sha256: "e881df011fa9711be90f1fea101a3391b5252d4daa61184fad9c75ca9032452f",
        maximum_sample_frames: LONG_AUDIO_MAXIMUM_FRAMES,
        sample_frames: 395_383,
    },
    FrozenObject {
        object_id: "metal-m9",
        object_name: "Metal 4",
        material_label: "Metal",
        recording_id: "m9",
        source_file_name: "AST_m9_expe.wav",
        track_number: 4,
        maximum_bytes: LONG_AUDIO_MAXIMUM_BYTES,
        bytes: 590_024,
        sha256: "6401a97e7a68aa8d7de16a8286af46b8752cc971e0235d9b5810b57c8741732d",
        maximum_sample_frames: LONG_AUDIO_MAXIMUM_FRAMES,
        sample_frames: 292_964,
    },
    FrozenObject {
        object_id: "metal-m10",
        object_name: "Metal 5",
        material_label: "Metal",
        recording_id: "m10",
        source_file_name: "AST_m10_expe.wav",
        track_number: 5,
        maximum_bytes: LONG_AUDIO_MAXIMUM_BYTES,
        bytes: 483_340,
        sha256: "d12efe9861c1c3b513a694b6c30d2f97a86ee183712c49b56fdfb7ad378f1372",
        maximum_sample_frames: LONG_AUDIO_MAXIMUM_FRAMES,
        sample_frames: 239_622,
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
        || profile.material_label != expected.material_label
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
            maximum_frames: expected.maximum_sample_frames,
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
        None,
        expected.maximum_bytes,
        expected.bytes,
        expected.sha256,
    )?;
    let (normalization_policy, page_bytes, page_sha256) = project_page_contract(expected);
    require_artifact(
        artifact(source, "project-page")?,
        ArtifactRole::ProjectDescription,
        LANDING_PAGE_URL,
        normalization_policy,
        PROJECT_PAGE_MAXIMUM_BYTES,
        page_bytes,
        page_sha256,
    )?;
    Ok(())
}

fn require_artifact(
    artifact: &RemoteArtifact,
    role: ArtifactRole,
    url: &str,
    normalization_policy: Option<FetchNormalizationPolicy>,
    maximum_bytes: u64,
    expected_bytes: u64,
    expected_sha256: &str,
) -> Result<(), String> {
    if artifact.role != role
        || artifact.url != url
        || artifact.redirect_policy.is_some()
        || artifact.normalization_policy != normalization_policy
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

fn project_page_contract(
    expected: &FrozenObject,
) -> (Option<FetchNormalizationPolicy>, u64, &'static str) {
    if expected.material_label == "Glass" {
        (None, PROJECT_PAGE_BYTES, PROJECT_PAGE_SHA256)
    } else {
        (
            Some(FetchNormalizationPolicy::KronlandMaterialPageIdentityV1),
            NORMALIZED_PROJECT_PAGE_BYTES,
            NORMALIZED_PROJECT_PAGE_SHA256,
        )
    }
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
    if let Ok(identity) = serde_json::from_slice::<NormalizedPageIdentity>(bytes) {
        return validate_normalized_project_page(&identity, expected);
    }
    let page = std::str::from_utf8(bytes)
        .map_err(|error| format!("Kronland project page must be UTF-8: {error}"))?;
    let recorded_scope = "Impact sounds from everyday life objects made of Wood, Metal and Glass materials were recorded";
    let exact_track = format!(
        "<a href='{AUDIO_PREFIX}{}'>{} {} original</a>",
        expected.source_file_name, expected.material_label, expected.track_number
    );
    let synthesized_track = format!("AST_{}_synth.wav", expected.recording_id);
    let tuned_track = format!("AST_{}_transp.wav", expected.recording_id);
    if !page.contains("Controlling the perceived material in an impact sound synthesizer")
        || !page.contains("The Language of Sounds")
        || !page.contains(recorded_scope)
        || !page.contains(&exact_track)
        || !page.contains(&synthesized_track)
        || !page.contains(&tuned_track)
    {
        return Err(format!(
            "Kronland page does not bind {} to its original {} recording",
            expected.object_id, expected.material_label
        ));
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NormalizedPageIdentity {
    schema: String,
    source_url: String,
    title: String,
    authors: String,
    publication_date: String,
    journal: String,
    recorded_scope: String,
    tracks: Vec<NormalizedTrackIdentity>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NormalizedTrackIdentity {
    material_label: String,
    track_number: u8,
    original_file_name: String,
    original_url: String,
    synthesized_file_name: String,
    tuned_file_name: String,
}

fn validate_normalized_project_page(
    identity: &NormalizedPageIdentity,
    expected: &FrozenObject,
) -> Result<(), String> {
    let recorded_scope = "Impact sounds from everyday life objects made of Wood, Metal and Glass materials were recorded";
    if identity.schema != NORMALIZED_PROJECT_PAGE_SCHEMA
        || identity.source_url != LANDING_PAGE_URL
        || identity.title != "Controlling the perceived material in an impact sound synthesizer"
        || identity.authors != "Aramaki M., Besson M., Kronland-Martinet R., Ystad S."
        || identity.publication_date != "September 2011"
        || identity.journal != "IEEE Transactions on Audio, Speech, and Language Processing"
        || identity.recorded_scope != recorded_scope
        || identity.tracks.len() != 15
    {
        return Err("Kronland normalized project identity is not the frozen page".to_owned());
    }
    let expected_original_url = format!("{AUDIO_PREFIX}{}", expected.source_file_name);
    let expected_synthesized = format!("AST_{}_synth.wav", expected.recording_id);
    let expected_tuned = format!("AST_{}_transp.wav", expected.recording_id);
    if !identity.tracks.iter().any(|track| {
        track.material_label == expected.material_label
            && track.track_number == expected.track_number
            && track.original_file_name == expected.source_file_name
            && track.original_url == expected_original_url
            && track.synthesized_file_name == expected_synthesized
            && track.tuned_file_name == expected_tuned
    }) {
        return Err(format!(
            "Kronland normalized page does not bind {} to its original {} recording",
            expected.object_id, expected.material_label
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
    fn declaration_freezes_each_original_material_object() {
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
        *material_label = "Ceramic".to_owned();
        assert!(validate_declaration(&source).is_err());
    }

    #[test]
    fn page_binding_distinguishes_original_from_derived_tracks() {
        let expected = OBJECTS[0];
        let page = format!(
            "Controlling the perceived material in an impact sound synthesizer — The Language of Sounds\n\
             Impact sounds from everyday life objects made of Wood, Metal and Glass materials were recorded\n\
             <a href='{AUDIO_PREFIX}{}'>{} {} original</a>\n\
             AST_{}_synth.wav AST_{}_transp.wav",
            expected.source_file_name,
            expected.material_label,
            expected.track_number,
            expected.recording_id,
            expected.recording_id
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
        let (normalization_policy, page_bytes, page_sha256) = project_page_contract(&expected);
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
                material_label: expected.material_label.to_owned(),
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
                    maximum_bytes: expected.maximum_bytes,
                    expected_byte_count: Some(expected.bytes),
                    expected_sha256: Some(expected.sha256.to_owned()),
                },
                RemoteArtifact {
                    id: "project-page".to_owned(),
                    role: ArtifactRole::ProjectDescription,
                    url: LANDING_PAGE_URL.to_owned(),
                    redirect_policy: None,
                    normalization_policy,
                    maximum_bytes: PROJECT_PAGE_MAXIMUM_BYTES,
                    expected_byte_count: Some(page_bytes),
                    expected_sha256: Some(page_sha256.to_owned()),
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
