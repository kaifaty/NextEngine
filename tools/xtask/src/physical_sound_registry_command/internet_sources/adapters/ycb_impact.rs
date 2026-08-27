use std::collections::BTreeSet;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{
    AdapterAudit, AdapterEvidenceReport, AdapterProfile, ArtifactRole, EvidenceCapability,
    InternetSource, RecordingReport, RemoteArtifact, wav,
};
use crate::physical_sound_registry_command::internet_sources::{
    FetchRedirectPolicy, RedistributionPolicy,
};

mod vertical;

pub(super) const ADAPTER_ID: &str = "ycb-impact-identified-recording-v1";

const PUBLISHER_ID: &str = "iri-csic-upc-ctu";
const PROJECT_ID: &str = "ycb-impact-sounds";
const DECLARED_REVISION: &str = "osf-bj5w8-2022-09-27";
const LANDING_PAGE_URL: &str = "https://osf.io/4tcp6/";
const OSF_STORAGE_PREFIX: &str = "https://files.de-1.osf.io/v1/resources";
const METADATA_FILE_ID: &str = "62330db6e919450662177fa3";
const METADATA_SHA256: &str = "27672ecfdfaf2a1ecfc8926127ab9ab962e9cb59adb593b141caadc0459ae0fd";
const METADATA_BYTES: u64 = 7_994;
const METADATA_MAXIMUM_BYTES: u64 = 32 * 1024;
const WAVE_RECORDING_BYTES: u64 = 1_920_058;
const WAVE_RECORDING_MAXIMUM_BYTES: u64 = 2 * 1024 * 1024;
const WAVE_RECORDING_FRAMES: u64 = 240_000;
const OGG_RECORDING_MAXIMUM_BYTES: u64 = 1024 * 1024;
const OGG_RECORDING_MAXIMUM_FRAMES: u64 = 44_100 * 60;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::physical_sound_registry_command::internet_sources) struct RecordingProfile {
    pub(super) condition_id: String,
    pub(super) clip_id: String,
    pub(super) osf_file_id: String,
}

#[derive(Serialize)]
pub(in crate::physical_sound_registry_command::internet_sources) struct RecordingEvidenceReport {
    pub(in crate::physical_sound_registry_command::internet_sources) condition_id: String,
    pub(in crate::physical_sound_registry_command::internet_sources) clip_id: String,
    pub(in crate::physical_sound_registry_command::internet_sources) osf_file_id: String,
    pub(in crate::physical_sound_registry_command::internet_sources) source_file_name: &'static str,
    pub(in crate::physical_sound_registry_command::internet_sources) container_format: &'static str,
    #[serde(flatten)]
    pub(in crate::physical_sound_registry_command::internet_sources) audio: RecordingReport,
}

#[derive(Clone, Copy)]
enum AudioFormat {
    FloatWave,
    OggVorbis,
}

#[derive(Clone, Copy)]
struct FrozenRecording {
    condition_id: &'static str,
    clip_id: &'static str,
    osf_file_id: &'static str,
    source_file_name: &'static str,
    expected_bytes: u64,
    sha256: &'static str,
    audio_format: AudioFormat,
}

struct FrozenObject {
    source_id: &'static str,
    object_id: &'static str,
    object_name: &'static str,
    primary_material_label: &'static str,
    secondary_material_label: Option<&'static str>,
    source_split: &'static str,
    recordings: &'static [FrozenRecording],
}

const WINEGLASS_RECORDINGS: [FrozenRecording; 4] = [
    FrozenRecording {
        condition_id: "horizontal-0_14",
        clip_id: "001",
        osf_file_id: "622a10751e399c0b23600d31",
        source_file_name: "Clip_1.ogg",
        expected_bytes: WAVE_RECORDING_BYTES,
        sha256: "56d6b75ac43c59d9b934a0f856f6e41b53c26deb5d4ccea131cd791239c07149",
        audio_format: AudioFormat::FloatWave,
    },
    FrozenRecording {
        condition_id: "horizontal-0_14",
        clip_id: "002",
        osf_file_id: "622a10761e399c0b25600eb9",
        source_file_name: "Clip_2.ogg",
        expected_bytes: WAVE_RECORDING_BYTES,
        sha256: "3ab969cca8eee925ed4f02017ceb3cda3615fc458eb135e809fe617b4108e619",
        audio_format: AudioFormat::FloatWave,
    },
    FrozenRecording {
        condition_id: "horizontal-0_25",
        clip_id: "001",
        osf_file_id: "622a10cb4ef4bb0a42334494",
        source_file_name: "Clip_1.ogg",
        expected_bytes: WAVE_RECORDING_BYTES,
        sha256: "f77e2556cd334f85b562476cf13a19659496cfa90cf7594a7b2c47a55f76d6bd",
        audio_format: AudioFormat::FloatWave,
    },
    FrozenRecording {
        condition_id: "horizontal-0_25",
        clip_id: "002",
        osf_file_id: "622a10c7d3db1309dd7e64d9",
        source_file_name: "Clip_2.ogg",
        expected_bytes: WAVE_RECORDING_BYTES,
        sha256: "ecb1f9a5aeaedce076c010ce56782c2a42e9bfdf031e481ea59ae4df0e134a27",
        audio_format: AudioFormat::FloatWave,
    },
];

const SKILLET_LID_RECORDINGS: [FrozenRecording; 4] = [
    FrozenRecording {
        condition_id: "horizontal-0_14",
        clip_id: "001",
        osf_file_id: "622a0e50558e6009a32a1093",
        source_file_name: "Clip_1.ogg",
        expected_bytes: WAVE_RECORDING_BYTES,
        sha256: "85afaad16436cd2c8d3a07074842df66e5bd0986b41b4ba0ededc3dc9b9a30d2",
        audio_format: AudioFormat::FloatWave,
    },
    FrozenRecording {
        condition_id: "horizontal-0_14",
        clip_id: "002",
        osf_file_id: "622a0e504ef4bb0a3d334530",
        source_file_name: "Clip_2.ogg",
        expected_bytes: WAVE_RECORDING_BYTES,
        sha256: "d1ff5bf764394ea41c6e73eed5c0b66b33ff301eab85eaab316a5e64db5ee975",
        audio_format: AudioFormat::FloatWave,
    },
    FrozenRecording {
        condition_id: "horizontal-0_25",
        clip_id: "001",
        osf_file_id: "622a0e7cd3db1309da7e64f8",
        source_file_name: "Clip_1.ogg",
        expected_bytes: WAVE_RECORDING_BYTES,
        sha256: "a94701d95ceb624e4647c08a9aaf4d89499e4d83591ee22553652e0f0621ac9c",
        audio_format: AudioFormat::FloatWave,
    },
    FrozenRecording {
        condition_id: "horizontal-0_25",
        clip_id: "002",
        osf_file_id: "622a0e7dd3db1309da7e64fc",
        source_file_name: "Clip_2.ogg",
        expected_bytes: WAVE_RECORDING_BYTES,
        sha256: "5ad7fa2a6d1429042eb7933ba0028f5df051ba1f2384fc8e3e13ca72cab55565",
        audio_format: AudioFormat::FloatWave,
    },
];

const WINEGLASS: FrozenObject = FrozenObject {
    source_id: "ycb-impact-object-023-wineglass",
    object_id: "23",
    object_name: "Wineglass",
    primary_material_label: "Glass",
    secondary_material_label: None,
    source_split: "train",
    recordings: &WINEGLASS_RECORDINGS,
};

const SKILLET_LID: FrozenObject = FrozenObject {
    source_id: "ycb-impact-object-028-skillet-lid",
    object_id: "28",
    object_name: "Skillet lid",
    primary_material_label: "Glass",
    secondary_material_label: Some("Hard Plastic"),
    source_split: "test",
    recordings: &SKILLET_LID_RECORDINGS,
};

pub(super) fn validate_declaration(source: &InternetSource) -> Result<(), String> {
    let AdapterProfile::YcbImpactIdentifiedRecordingV1 {
        object_id,
        object_name,
        primary_material_label,
        secondary_material_label,
        source_split,
        recordings,
    } = source
        .adapter_profile
        .as_ref()
        .ok_or_else(|| format!("source {} has no YCB Impact profile", source.id))?
    else {
        return Err(format!(
            "source {} has the wrong YCB Impact profile",
            source.id
        ));
    };
    let frozen = frozen_object(object_id)?;
    if source.id != frozen.source_id
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
        || object_name != frozen.object_name
        || primary_material_label != frozen.primary_material_label
        || secondary_material_label.as_deref() != frozen.secondary_material_label
        || source_split != frozen.source_split
    {
        return Err(format!(
            "source {} does not match the frozen official YCB Impact object identity",
            source.id
        ));
    }
    validate_recording_profile(recordings, frozen)?;
    validate_artifacts(source, frozen)?;
    validate_capability_evidence(source)?;
    Ok(())
}

pub(super) fn audit(cache: &Path, source: &InternetSource) -> Result<AdapterAudit, String> {
    let AdapterProfile::YcbImpactIdentifiedRecordingV1 {
        object_id,
        object_name,
        primary_material_label,
        secondary_material_label,
        source_split,
        ..
    } = source
        .adapter_profile
        .as_ref()
        .ok_or_else(|| format!("source {} has no YCB Impact profile", source.id))?
    else {
        return Err(format!(
            "source {} has the wrong YCB Impact profile",
            source.id
        ));
    };
    let frozen = frozen_object(object_id)?;
    let metadata = super::read_cached(cache, artifact(source, "object-metadata")?)?;
    if metadata.len() as u64 != METADATA_BYTES || !metadata.starts_with(b"PK\x03\x04") {
        return Err("YCB Impact object metadata is not the pinned XLSX payload".to_owned());
    }
    let mut recordings = Vec::with_capacity(frozen.recordings.len());
    for expected in frozen.recordings {
        let recording_id = recording_id(expected);
        let bytes = super::read_cached(cache, artifact(source, &artifact_id(expected))?)?;
        let (container_format, audio) = match expected.audio_format {
            AudioFormat::FloatWave => {
                let audio = wav::validate_float_wav(
                    &recording_id,
                    &bytes,
                    wav::FloatWavExpectation {
                        source_label: "YCB Impact recording",
                        sample_rate_hz: 48_000,
                        channel_count: 2,
                        maximum_frames: WAVE_RECORDING_FRAMES,
                    },
                )?;
                if audio.sample_frames != WAVE_RECORDING_FRAMES {
                    return Err(format!(
                        "YCB Impact recording {recording_id} must contain exactly {WAVE_RECORDING_FRAMES} frames"
                    ));
                }
                ("riff_wave", audio)
            }
            AudioFormat::OggVorbis => (
                "ogg",
                super::ogg_vorbis::validate_vorbis_in_ogg(
                    &recording_id,
                    &bytes,
                    super::ogg_vorbis::Expectation {
                        source_label: "YCB Impact vertical recording",
                        sample_rate_hz: 44_100,
                        channel_count: 2,
                        maximum_sample_frames: OGG_RECORDING_MAXIMUM_FRAMES,
                    },
                )?,
            ),
        };
        recordings.push(RecordingEvidenceReport {
            condition_id: expected.condition_id.to_owned(),
            clip_id: expected.clip_id.to_owned(),
            osf_file_id: expected.osf_file_id.to_owned(),
            source_file_name: expected.source_file_name,
            container_format,
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
        evidence: Some(AdapterEvidenceReport::YcbImpactIdentifiedRecordingV1 {
            object_id: object_id.clone(),
            object_name: object_name.clone(),
            primary_material_label: primary_material_label.clone(),
            secondary_material_label: secondary_material_label.clone(),
            source_split: source_split.clone(),
            recordings,
        }),
    })
}

fn frozen_object(object_id: &str) -> Result<&'static FrozenObject, String> {
    match object_id {
        "23" => Ok(&WINEGLASS),
        "28" => Ok(&SKILLET_LID),
        _ => vertical::OBJECTS
            .iter()
            .find(|object| object.object_id == object_id)
            .ok_or_else(|| format!("YCB Impact adapter v1 does not freeze object {object_id}")),
    }
}

fn validate_recording_profile(
    actual: &[RecordingProfile],
    frozen: &FrozenObject,
) -> Result<(), String> {
    if actual.len() != frozen.recordings.len()
        || actual
            .iter()
            .zip(frozen.recordings)
            .any(|(actual, expected)| {
                actual.condition_id != expected.condition_id
                    || actual.clip_id != expected.clip_id
                    || actual.osf_file_id != expected.osf_file_id
            })
    {
        return Err(format!(
            "YCB Impact object {} recording identities do not match adapter v1",
            frozen.object_id
        ));
    }
    Ok(())
}

fn validate_artifacts(source: &InternetSource, frozen: &FrozenObject) -> Result<(), String> {
    if source.artifacts.len() != frozen.recordings.len() + 1 {
        return Err(format!(
            "source {} must contain {} recordings and one metadata workbook",
            source.id,
            frozen.recordings.len()
        ));
    }
    require_artifact(
        artifact(source, "object-metadata")?,
        ArtifactRole::Metadata,
        &format!("{OSF_STORAGE_PREFIX}/4tcp6/providers/osfstorage/{METADATA_FILE_ID}"),
        METADATA_MAXIMUM_BYTES,
        METADATA_BYTES,
        METADATA_SHA256,
    )?;
    for recording in frozen.recordings {
        let maximum_bytes = match recording.audio_format {
            AudioFormat::FloatWave => WAVE_RECORDING_MAXIMUM_BYTES,
            AudioFormat::OggVorbis => OGG_RECORDING_MAXIMUM_BYTES,
        };
        require_artifact(
            artifact(source, &artifact_id(recording))?,
            ArtifactRole::AudioPayload,
            &format!(
                "{OSF_STORAGE_PREFIX}/bj5w8/providers/osfstorage/{}",
                recording.osf_file_id
            ),
            maximum_bytes,
            recording.expected_bytes,
            recording.sha256,
        )?;
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
            "source {} must bind every E3 capability to all frozen YCB artifacts",
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
        .ok_or_else(|| format!("source {} is missing YCB Impact artifact {id}", source.id))
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
        || artifact.redirect_policy != Some(FetchRedirectPolicy::OsfStorageV1)
        || artifact.maximum_bytes != maximum_bytes
        || artifact.expected_byte_count != Some(expected_bytes)
        || artifact.expected_sha256.as_deref() != Some(expected_sha256)
    {
        return Err(format!(
            "YCB Impact artifact {} does not match adapter v1",
            artifact.id
        ));
    }
    Ok(())
}

fn artifact_id(recording: &FrozenRecording) -> String {
    format!("impact-{}-{}", recording.condition_id, recording.clip_id)
}

fn recording_id(recording: &FrozenRecording) -> String {
    format!("{}-{}", recording.condition_id, recording.clip_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physical_sound_registry_command::FileRef;
    use crate::physical_sound_registry_command::internet_sources::{
        CapabilityEvidence, RemoteArtifact,
    };

    #[test]
    fn frozen_objects_keep_distinct_upstream_splits() {
        assert_eq!(
            frozen_object("23").expect("wineglass").source_split,
            "train"
        );
        assert_eq!(
            frozen_object("28").expect("skillet lid").source_split,
            "test"
        );
        assert!(frozen_object("49").is_err());
    }

    #[test]
    fn vertical_selection_is_balanced_unique_and_non_glass() {
        assert_eq!(vertical::OBJECTS.len(), 27);
        let material_counts = vertical::OBJECTS.iter().fold(
            std::collections::BTreeMap::<&str, usize>::new(),
            |mut counts, object| {
                *counts.entry(object.primary_material_label).or_default() += 1;
                counts
            },
        );
        assert_eq!(material_counts.len(), 9);
        assert!(material_counts.values().all(|count| *count == 3));
        assert!(!material_counts.contains_key("Glass"));
        assert!(
            vertical::OBJECTS
                .iter()
                .all(|object| object.recordings.len() == 2)
        );
        let recording_hashes = vertical::OBJECTS
            .iter()
            .flat_map(|object| object.recordings)
            .map(|recording| recording.sha256)
            .collect::<BTreeSet<_>>();
        assert_eq!(recording_hashes.len(), 54);
    }

    #[test]
    fn ycb_audio_requires_stereo_48khz_float_wave() {
        let bytes = wav::test_float_wav(48_000, 2, &[0.25, -0.25, 0.5, -0.5]);
        let report = wav::validate_float_wav(
            "test",
            &bytes,
            wav::FloatWavExpectation {
                source_label: "YCB Impact recording",
                sample_rate_hz: 48_000,
                channel_count: 2,
                maximum_frames: 2,
            },
        )
        .expect("valid YCB source format");
        assert_eq!(report.sample_frames, 2);
        assert_eq!(report.channel_count, 2);
        assert!(
            wav::validate_float_wav(
                "test",
                &bytes,
                wav::FloatWavExpectation {
                    source_label: "YCB Impact recording",
                    sample_rate_hz: 48_000,
                    channel_count: 1,
                    maximum_frames: 4,
                },
            )
            .is_err()
        );
    }

    #[test]
    fn declaration_requires_the_frozen_object_files_and_redirect_policy() {
        let mut source = test_source(&WINEGLASS);
        validate_declaration(&source).expect("frozen YCB declaration");
        source.artifacts[0].redirect_policy = None;
        assert!(validate_declaration(&source).is_err());

        let mut source = test_source(&SKILLET_LID);
        let Some(AdapterProfile::YcbImpactIdentifiedRecordingV1 {
            secondary_material_label,
            ..
        }) = source.adapter_profile.as_mut()
        else {
            panic!("YCB profile");
        };
        *secondary_material_label = None;
        assert!(validate_declaration(&source).is_err());

        let mut source = test_source(&vertical::OBJECTS[0]);
        validate_declaration(&source).expect("frozen vertical YCB declaration");
        let Some(AdapterProfile::YcbImpactIdentifiedRecordingV1 {
            primary_material_label,
            ..
        }) = source.adapter_profile.as_mut()
        else {
            panic!("YCB profile");
        };
        *primary_material_label = "Glass".to_owned();
        assert!(validate_declaration(&source).is_err());
    }

    fn test_source(frozen: &FrozenObject) -> InternetSource {
        let mut artifacts = frozen
            .recordings
            .iter()
            .map(|recording| RemoteArtifact {
                id: artifact_id(recording),
                role: ArtifactRole::AudioPayload,
                url: format!(
                    "{OSF_STORAGE_PREFIX}/bj5w8/providers/osfstorage/{}",
                    recording.osf_file_id
                ),
                redirect_policy: Some(FetchRedirectPolicy::OsfStorageV1),
                normalization_policy: None,
                maximum_bytes: match recording.audio_format {
                    AudioFormat::FloatWave => WAVE_RECORDING_MAXIMUM_BYTES,
                    AudioFormat::OggVorbis => OGG_RECORDING_MAXIMUM_BYTES,
                },
                expected_byte_count: Some(recording.expected_bytes),
                expected_sha256: Some(recording.sha256.to_owned()),
            })
            .collect::<Vec<_>>();
        artifacts.push(RemoteArtifact {
            id: "object-metadata".to_owned(),
            role: ArtifactRole::Metadata,
            url: format!("{OSF_STORAGE_PREFIX}/4tcp6/providers/osfstorage/{METADATA_FILE_ID}"),
            redirect_policy: Some(FetchRedirectPolicy::OsfStorageV1),
            normalization_policy: None,
            maximum_bytes: METADATA_MAXIMUM_BYTES,
            expected_byte_count: Some(METADATA_BYTES),
            expected_sha256: Some(METADATA_SHA256.to_owned()),
        });
        let artifact_ids = artifacts
            .iter()
            .map(|artifact| artifact.id.clone())
            .collect::<Vec<_>>();
        InternetSource {
            id: frozen.source_id.to_owned(),
            publisher_id: PUBLISHER_ID.to_owned(),
            project_id: PROJECT_ID.to_owned(),
            declared_revision: DECLARED_REVISION.to_owned(),
            review_date: "2026-08-27".to_owned(),
            landing_page_url: LANDING_PAGE_URL.to_owned(),
            terms_url: None,
            adapter_id: ADAPTER_ID.to_owned(),
            adapter_profile: Some(AdapterProfile::YcbImpactIdentifiedRecordingV1 {
                object_id: frozen.object_id.to_owned(),
                object_name: frozen.object_name.to_owned(),
                primary_material_label: frozen.primary_material_label.to_owned(),
                secondary_material_label: frozen.secondary_material_label.map(str::to_owned),
                source_split: frozen.source_split.to_owned(),
                recordings: frozen
                    .recordings
                    .iter()
                    .map(|recording| RecordingProfile {
                        condition_id: recording.condition_id.to_owned(),
                        clip_id: recording.clip_id.to_owned(),
                        osf_file_id: recording.osf_file_id.to_owned(),
                    })
                    .collect(),
            }),
            license_expression: "NOASSERTION".to_owned(),
            redistribution_policy: RedistributionPolicy::ExternalResearchOnly,
            provenance_review: FileRef {
                path: "provenance.md".to_owned(),
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
