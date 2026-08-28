use std::collections::BTreeSet;
use std::path::Path;

use serde::Deserialize;

use super::{
    AdapterAudit, AdapterEvidenceReport, AdapterProfile, ArtifactRole, EvidenceCapability,
    InternetSource, RemoteArtifact, rar_archive, wav,
};
use crate::physical_sound_registry_command::internet_sources::{
    FetchNormalizationPolicy, FetchRedirectPolicy, RedistributionPolicy,
};

pub(super) const ADAPTER_ID: &str = "soundpacks-glass-recordings-identified-recording-v1";

const PUBLISHER_ID: &str = "kaffekrus";
const PROJECT_ID: &str = "soundpacks-glass-recordings";
const DECLARED_REVISION: &str = "soundpacks-post-384-mediafire-nxuj8iiakqecpnu-2026-08-28";
const LANDING_PAGE_URL: &str = "https://soundpacks.com/free-sound-packs/glass-recordings/";
const ARCHIVE_URL: &str =
    "https://www.mediafire.com/file/nxuj8iiakqecpnu/Glass_Recordings_by_kaffekrus.rar/file";
const LICENSE_EXPRESSION: &str =
    "LicenseRef-kaffekrus-Glass-Recordings-Royalty-Free-No-Redistribution";
const MATERIAL_LABEL: &str = "Glass";
const PACK_IDENTITY_SCHEMA: &str =
    "nextengine.experimental-soundpacks-glass-recordings-identity.v1";
const PACK_IDENTITY_MAXIMUM_TRANSFER_BYTES: u64 = 128 * 1024;
const PACK_IDENTITY_BYTES: u64 = 734;
const PACK_IDENTITY_SHA256: &str =
    "4158f8c6e263c148cbf0539fee52ab45d2fd081a4233a48b2c28b5a153bbd5d5";
const ARCHIVE_MAXIMUM_BYTES: u64 = 40 * 1024 * 1024;
const ARCHIVE_BYTES: u64 = 35_155_966;
const ARCHIVE_SHA256: &str = "ee4e64741b33f678fdd6d1537b53bcb66417393ec233bae5531da61cef5a9aec";
const README_ENTRY: &str = "Glass Recordings by kaffekrus/readme.txt";
const README_BYTES: u64 = 447;
const README_SHA256: &str = "2b3322e8408f2b956dd9362176dad25f25d0836c3029f12d98532bb96bac72fb";
const MAX_RECORDING_FRAMES: u64 = 150_000;

#[derive(Clone, Debug, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::physical_sound_registry_command::internet_sources) struct RecordingProfile {
    pub(super) repeat_id: String,
    pub(super) archive_entry: String,
}

#[derive(serde::Serialize)]
pub(in crate::physical_sound_registry_command::internet_sources) struct RecordingEvidenceReport {
    pub(in crate::physical_sound_registry_command::internet_sources) repeat_id: String,
    pub(in crate::physical_sound_registry_command::internet_sources) archive_entry: &'static str,
    pub(in crate::physical_sound_registry_command::internet_sources) audio_file_bytes: u64,
    pub(in crate::physical_sound_registry_command::internet_sources) audio_file_sha256:
        &'static str,
    pub(in crate::physical_sound_registry_command::internet_sources) audio: super::RecordingReport,
}

#[derive(Clone, Copy)]
struct FrozenRecording {
    repeat_id: &'static str,
    archive_entry: &'static str,
    bytes: u64,
    sha256: &'static str,
    sample_frames: u64,
}

#[derive(Clone, Copy)]
struct FrozenObject {
    object_id: &'static str,
    object_name: &'static str,
    recordings: &'static [FrozenRecording],
}

const DRINKING_GLASS_RECORDINGS: [FrozenRecording; 4] = [
    FrozenRecording {
        repeat_id: "001",
        archive_entry: "Glass Recordings by kaffekrus/hits/drinking glass1.wav",
        bytes: 1_058_444,
        sha256: "f34feec9c1dff776a1bb05b0ff938a76db2027d1926da43343554620e50858bd",
        sample_frames: 132_300,
    },
    FrozenRecording {
        repeat_id: "002",
        archive_entry: "Glass Recordings by kaffekrus/hits/drinking glass2.wav",
        bytes: 917_324,
        sha256: "9ae504a6e930604a8e6bb6d291c4240c6bafe835249d504cd7e4f937c5c390c1",
        sample_frames: 114_660,
    },
    FrozenRecording {
        repeat_id: "003",
        archive_entry: "Glass Recordings by kaffekrus/hits/drinking glass3.wav",
        bytes: 564_524,
        sha256: "6787469f28e37668d73416177091909a9ed4702de89e85e794b7817d0ca29f57",
        sample_frames: 70_560,
    },
    FrozenRecording {
        repeat_id: "004",
        archive_entry: "Glass Recordings by kaffekrus/hits/drinking glass4.wav",
        bytes: 811_484,
        sha256: "24cb4b07bfa34e3f20695ff4adc15e28de60b53d2962f69e7ed0ed2034fa42fa",
        sample_frames: 101_430,
    },
];

const METALLIC_VASE_RECORDINGS: [FrozenRecording; 3] = [
    FrozenRecording {
        repeat_id: "001",
        archive_entry: "Glass Recordings by kaffekrus/hits/metallic vase1.wav",
        bytes: 635_084,
        sha256: "3d1fcefa8864a2273e3d84cd16ca3c8e37dcd544cfbf9ada92e6bcf9811c5753",
        sample_frames: 79_380,
    },
    FrozenRecording {
        repeat_id: "002",
        archive_entry: "Glass Recordings by kaffekrus/hits/metallic vase2.wav",
        bytes: 564_524,
        sha256: "c45d456d5c5e89c1aae412a6d6c735c4f1c5c9a684a994244166553778f25a3c",
        sample_frames: 70_560,
    },
    FrozenRecording {
        repeat_id: "003",
        archive_entry: "Glass Recordings by kaffekrus/hits/metallic vase3.wav",
        bytes: 952_604,
        sha256: "ccade1b659f723c69732c14f78770cf6ccda65245f0ac203d3f872bd8e95cd0f",
        sample_frames: 119_070,
    },
];

const OBJECTS: [FrozenObject; 2] = [
    FrozenObject {
        object_id: "drinking-glass",
        object_name: "Drinking glass",
        recordings: &DRINKING_GLASS_RECORDINGS,
    },
    FrozenObject {
        object_id: "metallic-vase",
        object_name: "Metallic-sounding glass vase",
        recordings: &METALLIC_VASE_RECORDINGS,
    },
];

pub(super) fn validate_declaration(source: &InternetSource) -> Result<(), String> {
    let (profile, expected) = profile(source)?;
    if source.id != format!("soundpacks-glass-recordings-{}", expected.object_id)
        || source.publisher_id != PUBLISHER_ID
        || source.project_id != PROJECT_ID
        || source.declared_revision != DECLARED_REVISION
        || source.review_date != "2026-08-28"
        || source.landing_page_url != LANDING_PAGE_URL
        || source.terms_url.is_some()
        || source.license_expression != LICENSE_EXPRESSION
        || !matches!(
            source.redistribution_policy,
            RedistributionPolicy::ExternalResearchOnly
        )
        || profile.object_name != expected.object_name
        || profile.material_label != MATERIAL_LABEL
    {
        return Err(format!(
            "source {} does not match the frozen SoundPacks Glass Recordings identity",
            source.id
        ));
    }
    validate_recording_profile(profile.recordings, expected)?;
    validate_artifacts(source)?;
    validate_capability_evidence(source)?;
    Ok(())
}

pub(super) fn audit(cache: &Path, source: &InternetSource) -> Result<AdapterAudit, String> {
    let (profile, expected) = profile(source)?;
    let identity_bytes = super::read_cached(cache, artifact(source, "pack-identity")?)?;
    validate_pack_identity(&identity_bytes)?;
    let archive = super::read_cached(cache, artifact(source, "audio-archive")?)?;
    let expectations = std::iter::once(rar_archive::ExpectedEntry {
        name: README_ENTRY,
        bytes: README_BYTES,
        sha256: README_SHA256,
    })
    .chain(
        expected
            .recordings
            .iter()
            .map(|recording| rar_archive::ExpectedEntry {
                name: recording.archive_entry,
                bytes: recording.bytes,
                sha256: recording.sha256,
            }),
    )
    .collect::<Vec<_>>();
    let extracted = rar_archive::read_exact_entries(&archive, &expectations)?;
    validate_readme(&extracted[0])?;
    let mut recordings = Vec::with_capacity(expected.recordings.len());
    for (recording, bytes) in expected.recordings.iter().zip(&extracted[1..]) {
        let audio = wav::validate_float_wav(
            recording.repeat_id,
            bytes,
            wav::FloatWavExpectation {
                source_label: "SoundPacks Glass Recordings impact",
                sample_rate_hz: 44_100,
                channel_count: 2,
                maximum_frames: MAX_RECORDING_FRAMES,
                require_fact_frames: false,
            },
        )?;
        if audio.sample_frames != recording.sample_frames {
            return Err(format!(
                "SoundPacks recording {} must contain exactly {} frames",
                recording.repeat_id, recording.sample_frames
            ));
        }
        recordings.push(RecordingEvidenceReport {
            repeat_id: recording.repeat_id.to_owned(),
            archive_entry: recording.archive_entry,
            audio_file_bytes: recording.bytes,
            audio_file_sha256: recording.sha256,
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
            AdapterEvidenceReport::SoundpacksGlassRecordingsIdentifiedRecordingV1 {
                object_id: profile.object_id.to_owned(),
                object_name: profile.object_name.to_owned(),
                material_label: profile.material_label.to_owned(),
                recordings,
            },
        ),
    })
}

struct Profile<'a> {
    object_id: &'a str,
    object_name: &'a str,
    material_label: &'a str,
    recordings: &'a [RecordingProfile],
}

fn profile(source: &InternetSource) -> Result<(Profile<'_>, &'static FrozenObject), String> {
    let AdapterProfile::SoundpacksGlassRecordingsIdentifiedRecordingV1 {
        object_id,
        object_name,
        material_label,
        recordings,
    } = source
        .adapter_profile
        .as_ref()
        .ok_or_else(|| format!("source {} has no SoundPacks profile", source.id))?
    else {
        return Err(format!(
            "source {} has the wrong SoundPacks profile",
            source.id
        ));
    };
    let expected = OBJECTS
        .iter()
        .find(|object| object.object_id == object_id)
        .ok_or_else(|| {
            format!(
                "source {} names an unsupported SoundPacks object",
                source.id
            )
        })?;
    Ok((
        Profile {
            object_id,
            object_name,
            material_label,
            recordings,
        },
        expected,
    ))
}

fn validate_recording_profile(
    actual: &[RecordingProfile],
    expected: &FrozenObject,
) -> Result<(), String> {
    if actual.len() != expected.recordings.len()
        || actual
            .iter()
            .zip(expected.recordings)
            .any(|(actual, expected)| {
                actual.repeat_id != expected.repeat_id
                    || actual.archive_entry != expected.archive_entry
            })
    {
        return Err("SoundPacks recording identities do not match adapter v1".to_owned());
    }
    Ok(())
}

fn validate_artifacts(source: &InternetSource) -> Result<(), String> {
    if source.artifacts.len() != 2 {
        return Err(format!(
            "source {} must contain the normalized pack identity and RAR archive",
            source.id
        ));
    }
    let archive = artifact(source, "audio-archive")?;
    if archive.role != ArtifactRole::AudioArchive
        || archive.url != ARCHIVE_URL
        || archive.redirect_policy != Some(FetchRedirectPolicy::MediafireFileV1)
        || archive.normalization_policy.is_some()
        || archive.maximum_bytes != ARCHIVE_MAXIMUM_BYTES
        || archive.expected_byte_count != Some(ARCHIVE_BYTES)
        || archive.expected_sha256.as_deref() != Some(ARCHIVE_SHA256)
    {
        return Err("SoundPacks audio archive does not match adapter v1".to_owned());
    }
    let identity = artifact(source, "pack-identity")?;
    if identity.role != ArtifactRole::Metadata
        || identity.url != LANDING_PAGE_URL
        || identity.redirect_policy.is_some()
        || identity.normalization_policy
            != Some(FetchNormalizationPolicy::SoundpacksGlassRecordingsIdentityV1)
        || identity.maximum_bytes != PACK_IDENTITY_MAXIMUM_TRANSFER_BYTES
        || identity.expected_byte_count != Some(PACK_IDENTITY_BYTES)
        || identity.expected_sha256.as_deref() != Some(PACK_IDENTITY_SHA256)
    {
        return Err("SoundPacks normalized identity does not match adapter v1".to_owned());
    }
    Ok(())
}

fn validate_capability_evidence(source: &InternetSource) -> Result<(), String> {
    let artifact_ids = vec!["audio-archive".to_owned(), "pack-identity".to_owned()];
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
            "source {} must bind every E3 capability to the SoundPacks identity and archive",
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
        .ok_or_else(|| format!("source {} is missing SoundPacks artifact {id}", source.id))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PackIdentity {
    schema: String,
    source_url: String,
    creator: String,
    title: String,
    published_at: String,
    modified_at: String,
    description: [String; 3],
    sample_count: u16,
    ambient_sound_count: u16,
    glass_hit_count: u16,
    format: String,
    archive_url: String,
    archive_display_size: String,
}

fn validate_pack_identity(bytes: &[u8]) -> Result<(), String> {
    let identity: PackIdentity = serde_json::from_slice(bytes)
        .map_err(|error| format!("parse normalized SoundPacks identity: {error}"))?;
    if identity.schema != PACK_IDENTITY_SCHEMA
        || identity.source_url != LANDING_PAGE_URL
        || identity.creator != "kaffekrus"
        || identity.title != "Glass Recordings"
        || identity.published_at != "2024-01-12T04:52:04Z"
        || identity.modified_at != "2024-10-30T05:51:24Z"
        || identity.description
            != [
                "field recordings and organic sounds",
                "apartment windows, mirrors, drinking glasses, vases, and other household objects",
                "11 ambient sounds and 28 glass hits",
            ]
        || identity.sample_count != 39
        || identity.ambient_sound_count != 11
        || identity.glass_hit_count != 28
        || identity.format != "WAV"
        || identity.archive_url != ARCHIVE_URL
        || identity.archive_display_size != "33.53MB"
    {
        return Err("normalized SoundPacks identity does not match adapter v1".to_owned());
    }
    Ok(())
}

fn validate_readme(bytes: &[u8]) -> Result<(), String> {
    let readme = std::str::from_utf8(bytes)
        .map_err(|error| format!("SoundPacks readme must be UTF-8: {error}"))?;
    if !readme.contains("All sounds are free to use in any artistic and creative way")
        || !readme.contains("reselling, monetizing, copying or forgery of this pack is prohibited")
        || !readme.contains("recordings of different types of glass in my apartment")
        || !readme.contains("handheld recorders")
    {
        return Err(
            "SoundPacks readme does not bind the frozen recording and terms identity".to_owned(),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physical_sound_registry_command::FileRef;
    use crate::physical_sound_registry_command::internet_sources::CapabilityEvidence;

    #[test]
    fn declaration_freezes_both_glass_object_families() {
        for expected in OBJECTS {
            validate_declaration(&test_source(expected)).expect("frozen SoundPacks declaration");
        }
        let mut source = test_source(OBJECTS[0]);
        let Some(AdapterProfile::SoundpacksGlassRecordingsIdentifiedRecordingV1 {
            material_label,
            ..
        }) = source.adapter_profile.as_mut()
        else {
            panic!("SoundPacks profile");
        };
        *material_label = "Metal".to_owned();
        assert!(validate_declaration(&source).is_err());
    }

    #[test]
    fn readme_must_retain_recording_and_terms_claims() {
        let readme = b"All sounds are free to use in any artistic and creative way\n\
reselling, monetizing, copying or forgery of this pack is prohibited\n\
recordings of different types of glass in my apartment\n\
handheld recorders";
        validate_readme(readme).expect("frozen readme claims");
        assert!(validate_readme(&readme.replace_range_for_test()).is_err());
    }

    trait TestMutation {
        fn replace_range_for_test(&self) -> Vec<u8>;
    }

    impl TestMutation for [u8] {
        fn replace_range_for_test(&self) -> Vec<u8> {
            String::from_utf8(self.to_vec())
                .expect("ASCII fixture")
                .replace("different types of glass", "different types of metal")
                .into_bytes()
        }
    }

    fn test_source(expected: FrozenObject) -> InternetSource {
        let artifact_ids = vec!["audio-archive".to_owned(), "pack-identity".to_owned()];
        InternetSource {
            id: format!("soundpacks-glass-recordings-{}", expected.object_id),
            publisher_id: PUBLISHER_ID.to_owned(),
            project_id: PROJECT_ID.to_owned(),
            declared_revision: DECLARED_REVISION.to_owned(),
            review_date: "2026-08-28".to_owned(),
            landing_page_url: LANDING_PAGE_URL.to_owned(),
            terms_url: None,
            adapter_id: ADAPTER_ID.to_owned(),
            adapter_profile: Some(
                AdapterProfile::SoundpacksGlassRecordingsIdentifiedRecordingV1 {
                    object_id: expected.object_id.to_owned(),
                    object_name: expected.object_name.to_owned(),
                    material_label: MATERIAL_LABEL.to_owned(),
                    recordings: expected
                        .recordings
                        .iter()
                        .map(|recording| RecordingProfile {
                            repeat_id: recording.repeat_id.to_owned(),
                            archive_entry: recording.archive_entry.to_owned(),
                        })
                        .collect(),
                },
            ),
            license_expression: LICENSE_EXPRESSION.to_owned(),
            redistribution_policy: RedistributionPolicy::ExternalResearchOnly,
            provenance_review: FileRef {
                path: "provenance.md".to_owned(),
                sha256: "aa".repeat(32),
            },
            artifacts: vec![
                RemoteArtifact {
                    id: "audio-archive".to_owned(),
                    role: ArtifactRole::AudioArchive,
                    url: ARCHIVE_URL.to_owned(),
                    redirect_policy: Some(FetchRedirectPolicy::MediafireFileV1),
                    normalization_policy: None,
                    maximum_bytes: ARCHIVE_MAXIMUM_BYTES,
                    expected_byte_count: Some(ARCHIVE_BYTES),
                    expected_sha256: Some(ARCHIVE_SHA256.to_owned()),
                },
                RemoteArtifact {
                    id: "pack-identity".to_owned(),
                    role: ArtifactRole::Metadata,
                    url: LANDING_PAGE_URL.to_owned(),
                    redirect_policy: None,
                    normalization_policy: Some(
                        FetchNormalizationPolicy::SoundpacksGlassRecordingsIdentityV1,
                    ),
                    maximum_bytes: PACK_IDENTITY_MAXIMUM_TRANSFER_BYTES,
                    expected_byte_count: Some(PACK_IDENTITY_BYTES),
                    expected_sha256: Some(PACK_IDENTITY_SHA256.to_owned()),
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
