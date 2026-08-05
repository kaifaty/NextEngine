//! Audio scene presentation contracts (SPEC-08 baseline audio, SPEC-30 cue
//! boundary).
//!
//! `AudioSceneSnapshotV1` is an immutable per-tick presentation projection:
//! exactly one listener, the active emitter set and the event-derived one-shot
//! cue set for one committed simulation tick, plus the deterministic acoustic
//! facts gameplay hearing may consume. It is derived from committed
//! `DomainEvent` records, physics-snapshot poses and locked content; it never
//! becomes gameplay authority, and mixer/device state never appears here.

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::ids::{ContentHash, EventId, PersistentId, SchemaId};
use crate::manifest_jcs::{JcsValue, encode_canonical_jcs};
use crate::project::{AssetRevisionRefV1, domain_hash};

use super::{
    QuantizedPresentationTransformV1, asset_revision_value, hex_bytes, number, object, string,
    transform_value, validate_orientation,
};

pub const AUDIO_SCENE_SNAPSHOT_SCHEMA_VERSION: u32 = 1;
pub const AUDIO_SCENE_MAX_EMITTERS: usize = 1_024;
pub const AUDIO_SCENE_MAX_CUES: usize = 1_024;
pub const AUDIO_SCENE_MAX_ACOUSTIC_FACTS: usize = 1_024;

/// Closed loudness classification for deterministic acoustic facts and mixer
/// admission. Authored per clip binding; never derived from device state.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum AudioLoudnessClassV1 {
    Quiet = 0,
    Normal = 1,
    Loud = 2,
}

impl AudioLoudnessClassV1 {
    const fn token(self) -> &'static str {
        match self {
            Self::Quiet => "Quiet",
            Self::Normal => "Normal",
            Self::Loud => "Loud",
        }
    }
}

/// Closed priority classification for bounded voice admission. Higher
/// priority wins admission; ties break by canonical cue identity.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum AudioPriorityClassV1 {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl AudioPriorityClassV1 {
    const fn token(self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Normal => "Normal",
            Self::High => "High",
            Self::Critical => "Critical",
        }
    }
}

/// Engine-owned emitter identity: the durable subject the sound is attached
/// to plus an incarnation distinguishing destroy/recreate within one epoch.
/// Backend voices, mixer channels and array positions are never identity.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AudioEmitterKeyV1 {
    pub subject_id: PersistentId,
    pub incarnation: u32,
}

/// The single listener for one audio scene. The transform is the exact
/// quantized physics/presentation pose at the snapshot tick.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AudioListenerRecordV1 {
    pub listener_id: PersistentId,
    pub transform: QuantizedPresentationTransformV1,
}

impl AudioListenerRecordV1 {
    pub fn new(
        listener_id: PersistentId,
        transform: QuantizedPresentationTransformV1,
    ) -> Result<Self, AudioSceneContractErrorV1> {
        validate_orientation(transform.orientation_q30)
            .map_err(|_| AudioSceneContractErrorV1::InvalidOrientation)?;
        Ok(Self {
            listener_id,
            transform,
        })
    }
}

/// One active emitter in the scene: a clip bound to a positioned subject.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AudioEmitterRecordV1 {
    pub emitter_key: AudioEmitterKeyV1,
    pub clip_revision: AssetRevisionRefV1,
    pub transform: QuantizedPresentationTransformV1,
    pub loudness_class: AudioLoudnessClassV1,
    pub priority_class: AudioPriorityClassV1,
    pub occlusion_zone_or_none: Option<SchemaId>,
    pub looped: bool,
    pub canonical_hash: ContentHash,
}

impl AudioEmitterRecordV1 {
    pub fn new(
        emitter_key: AudioEmitterKeyV1,
        clip_revision: AssetRevisionRefV1,
        transform: QuantizedPresentationTransformV1,
        loudness_class: AudioLoudnessClassV1,
        priority_class: AudioPriorityClassV1,
        occlusion_zone_or_none: Option<SchemaId>,
        looped: bool,
    ) -> Result<Self, AudioSceneContractErrorV1> {
        validate_orientation(transform.orientation_q30)
            .map_err(|_| AudioSceneContractErrorV1::InvalidOrientation)?;
        if clip_revision.record_sha256 == ContentHash::default() {
            return Err(AudioSceneContractErrorV1::InvalidAssetRevision);
        }
        let mut value = Self {
            emitter_key,
            clip_revision,
            transform,
            loudness_class,
            priority_class,
            occlusion_zone_or_none,
            looped,
            canonical_hash: ContentHash::default(),
        };
        value.canonical_hash = value.computed_hash();
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), AudioSceneContractErrorV1> {
        validate_orientation(self.transform.orientation_q30)
            .map_err(|_| AudioSceneContractErrorV1::InvalidOrientation)?;
        if self.clip_revision.record_sha256 == ContentHash::default() {
            return Err(AudioSceneContractErrorV1::InvalidAssetRevision);
        }
        if self.computed_hash() != self.canonical_hash {
            return Err(AudioSceneContractErrorV1::HashMismatch);
        }
        Ok(())
    }

    fn computed_hash(&self) -> ContentHash {
        domain_hash(
            "nextengine.audio-emitter-record.v1",
            &encode_canonical_jcs(&object([
                ("clip_revision", asset_revision_value(self.clip_revision)),
                ("emitter_key", emitter_key_value(self.emitter_key)),
                ("looped", string(if self.looped { "true" } else { "false" })),
                ("loudness_class", string(self.loudness_class.token())),
                (
                    "occlusion_zone_or_none",
                    optional_zone_value(&self.occlusion_zone_or_none),
                ),
                ("priority_class", string(self.priority_class.token())),
                ("transform", transform_value(self.transform)),
            ])),
        )
    }
}

/// One event-derived one-shot audio activation. `cue_id` is engine-owned and
/// derived from the source event identity, the cue slot within that event and
/// the exact clip/emitter binding; re-extraction and replay derive the same
/// value. No backend voice or device identity participates.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AudioCueV1 {
    pub cue_id: ContentHash,
    pub source_event_id: EventId,
    pub source_cue_slot: u32,
    pub activation_simulation_tick: u64,
    pub emitter_key: AudioEmitterKeyV1,
    pub clip_revision: AssetRevisionRefV1,
    pub loudness_class: AudioLoudnessClassV1,
    pub priority_class: AudioPriorityClassV1,
    pub occlusion_zone_or_none: Option<SchemaId>,
    pub canonical_hash: ContentHash,
}

impl AudioCueV1 {
    #[allow(
        clippy::too_many_arguments,
        reason = "the closed cue record keeps every canonical identity field explicit"
    )]
    pub fn new(
        source_event_id: EventId,
        source_cue_slot: u32,
        activation_simulation_tick: u64,
        emitter_key: AudioEmitterKeyV1,
        clip_revision: AssetRevisionRefV1,
        loudness_class: AudioLoudnessClassV1,
        priority_class: AudioPriorityClassV1,
        occlusion_zone_or_none: Option<SchemaId>,
    ) -> Result<Self, AudioSceneContractErrorV1> {
        if clip_revision.record_sha256 == ContentHash::default() {
            return Err(AudioSceneContractErrorV1::InvalidAssetRevision);
        }
        let cue_id = derive_cue_id(source_event_id, source_cue_slot, emitter_key, clip_revision);
        let mut value = Self {
            cue_id,
            source_event_id,
            source_cue_slot,
            activation_simulation_tick,
            emitter_key,
            clip_revision,
            loudness_class,
            priority_class,
            occlusion_zone_or_none,
            canonical_hash: ContentHash::default(),
        };
        value.canonical_hash = value.computed_hash();
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), AudioSceneContractErrorV1> {
        if self.clip_revision.record_sha256 == ContentHash::default() {
            return Err(AudioSceneContractErrorV1::InvalidAssetRevision);
        }
        if self.cue_id
            != derive_cue_id(
                self.source_event_id,
                self.source_cue_slot,
                self.emitter_key,
                self.clip_revision,
            )
        {
            return Err(AudioSceneContractErrorV1::InvalidCueIdentity);
        }
        if self.computed_hash() != self.canonical_hash {
            return Err(AudioSceneContractErrorV1::HashMismatch);
        }
        Ok(())
    }

    fn computed_hash(&self) -> ContentHash {
        domain_hash(
            "nextengine.audio-cue-record.v1",
            &encode_canonical_jcs(&object([
                (
                    "activation_simulation_tick",
                    JcsValue::Number(self.activation_simulation_tick),
                ),
                ("clip_revision", asset_revision_value(self.clip_revision)),
                ("cue_id", string(self.cue_id.to_hex())),
                ("emitter_key", emitter_key_value(self.emitter_key)),
                ("loudness_class", string(self.loudness_class.token())),
                (
                    "occlusion_zone_or_none",
                    optional_zone_value(&self.occlusion_zone_or_none),
                ),
                ("priority_class", string(self.priority_class.token())),
                (
                    "source_cue_slot",
                    JcsValue::Number(u64::from(self.source_cue_slot)),
                ),
                (
                    "source_event_id",
                    string(hex_bytes(self.source_event_id.as_bytes())),
                ),
            ])),
        )
    }
}

/// Deterministic gameplay-hearing fact (SPEC-08 world services): one cue
/// activation rendered as an exact source/listener/loudness/zone/tick tuple.
/// It is independent of any output device or mixer state.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AcousticFactV1 {
    pub tick: u64,
    pub source: AudioEmitterKeyV1,
    pub listener_id: PersistentId,
    pub loudness_class: AudioLoudnessClassV1,
    pub occlusion_zone_or_none: Option<SchemaId>,
}

/// One immutable per-tick audio scene projection. Construction canonicalizes
/// order and rejects duplicates; the canonical hash binds the presentation
/// epoch/sequence and the simulation tick.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AudioSceneSnapshotV1 {
    pub schema_version: u32,
    pub snapshot_epoch: ContentHash,
    pub snapshot_sequence: u64,
    pub simulation_tick: u64,
    pub listener: AudioListenerRecordV1,
    pub emitters: Vec<AudioEmitterRecordV1>,
    pub cues: Vec<AudioCueV1>,
    pub acoustic_facts: Vec<AcousticFactV1>,
    pub canonical_hash: ContentHash,
}

impl AudioSceneSnapshotV1 {
    pub fn new(
        snapshot_epoch: ContentHash,
        snapshot_sequence: u64,
        simulation_tick: u64,
        listener: AudioListenerRecordV1,
        mut emitters: Vec<AudioEmitterRecordV1>,
        mut cues: Vec<AudioCueV1>,
        mut acoustic_facts: Vec<AcousticFactV1>,
    ) -> Result<Self, AudioSceneContractErrorV1> {
        if emitters.len() > AUDIO_SCENE_MAX_EMITTERS {
            return Err(AudioSceneContractErrorV1::LimitExceeded);
        }
        if cues.len() > AUDIO_SCENE_MAX_CUES
            || acoustic_facts.len() > AUDIO_SCENE_MAX_ACOUSTIC_FACTS
        {
            return Err(AudioSceneContractErrorV1::LimitExceeded);
        }
        emitters.sort_by_key(|record| record.emitter_key);
        cues.sort_by_key(cue_sort_key);
        acoustic_facts.sort();
        let value = Self {
            schema_version: AUDIO_SCENE_SNAPSHOT_SCHEMA_VERSION,
            snapshot_epoch,
            snapshot_sequence,
            simulation_tick,
            listener,
            emitters,
            cues,
            acoustic_facts,
            canonical_hash: ContentHash::default(),
        };
        value.ensure_unique_keys()?;
        let mut value = value;
        value.canonical_hash = value.computed_hash();
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), AudioSceneContractErrorV1> {
        if self.schema_version != AUDIO_SCENE_SNAPSHOT_SCHEMA_VERSION {
            return Err(AudioSceneContractErrorV1::UnsupportedVersion);
        }
        if self.emitters.len() > AUDIO_SCENE_MAX_EMITTERS
            || self.cues.len() > AUDIO_SCENE_MAX_CUES
            || self.acoustic_facts.len() > AUDIO_SCENE_MAX_ACOUSTIC_FACTS
        {
            return Err(AudioSceneContractErrorV1::LimitExceeded);
        }
        if self
            .emitters
            .windows(2)
            .any(|pair| pair[0].emitter_key >= pair[1].emitter_key)
            || self
                .cues
                .windows(2)
                .any(|pair| cue_sort_key(&pair[0]) >= cue_sort_key(&pair[1]))
            || self
                .acoustic_facts
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
        {
            return Err(AudioSceneContractErrorV1::NonCanonicalOrder);
        }
        self.ensure_unique_keys()?;
        for emitter in &self.emitters {
            emitter.validate()?;
        }
        for cue in &self.cues {
            cue.validate()?;
        }
        if self.computed_hash() != self.canonical_hash {
            return Err(AudioSceneContractErrorV1::HashMismatch);
        }
        Ok(())
    }

    fn ensure_unique_keys(&self) -> Result<(), AudioSceneContractErrorV1> {
        let mut emitter_keys = BTreeSet::new();
        for emitter in &self.emitters {
            if !emitter_keys.insert(emitter.emitter_key) {
                return Err(AudioSceneContractErrorV1::DuplicateEmitterKey);
            }
        }
        let mut cue_ids = BTreeSet::new();
        for cue in &self.cues {
            if !cue_ids.insert(cue.cue_id) {
                return Err(AudioSceneContractErrorV1::DuplicateCueId);
            }
        }
        Ok(())
    }

    fn computed_hash(&self) -> ContentHash {
        domain_hash(
            "nextengine.audio-scene-snapshot.v1",
            &encode_canonical_jcs(&object([
                (
                    "acoustic_facts",
                    JcsValue::Array(self.acoustic_facts.iter().map(fact_value).collect()),
                ),
                (
                    "cues",
                    JcsValue::Array(
                        self.cues
                            .iter()
                            .map(|cue| string(cue.canonical_hash.to_hex()))
                            .collect(),
                    ),
                ),
                (
                    "emitters",
                    JcsValue::Array(
                        self.emitters
                            .iter()
                            .map(|emitter| string(emitter.canonical_hash.to_hex()))
                            .collect(),
                    ),
                ),
                (
                    "listener",
                    object([
                        (
                            "listener_id",
                            string(hex_bytes(self.listener.listener_id.as_bytes())),
                        ),
                        ("transform", transform_value(self.listener.transform)),
                    ]),
                ),
                ("schema_version", number(self.schema_version)),
                ("simulation_tick", JcsValue::Number(self.simulation_tick)),
                ("snapshot_epoch", string(self.snapshot_epoch.to_hex())),
                (
                    "snapshot_sequence",
                    JcsValue::Number(self.snapshot_sequence),
                ),
            ])),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum AudioSceneContractErrorV1 {
    UnsupportedVersion,
    InvalidOrientation,
    InvalidAssetRevision,
    InvalidCueIdentity,
    DuplicateEmitterKey,
    DuplicateCueId,
    NonCanonicalOrder,
    LimitExceeded,
    HashMismatch,
}

impl Display for AudioSceneContractErrorV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::UnsupportedVersion => "audio scene schema version unsupported",
            Self::InvalidOrientation => "audio scene orientation is invalid",
            Self::InvalidAssetRevision => "audio scene asset revision hash is zero",
            Self::InvalidCueIdentity => "audio cue identity does not match its derivation",
            Self::DuplicateEmitterKey => "audio emitter key is duplicated",
            Self::DuplicateCueId => "audio cue ID is duplicated",
            Self::NonCanonicalOrder => "audio scene records are not canonically ordered",
            Self::LimitExceeded => "audio scene contract limit exceeded",
            Self::HashMismatch => "audio scene canonical hash mismatch",
        })
    }
}

impl Error for AudioSceneContractErrorV1 {}

fn derive_cue_id(
    source_event_id: EventId,
    source_cue_slot: u32,
    emitter_key: AudioEmitterKeyV1,
    clip_revision: AssetRevisionRefV1,
) -> ContentHash {
    domain_hash(
        "nextengine.audio-cue.v1",
        &encode_canonical_jcs(&object([
            ("clip_revision", asset_revision_value(clip_revision)),
            ("emitter_key", emitter_key_value(emitter_key)),
            (
                "source_cue_slot",
                JcsValue::Number(u64::from(source_cue_slot)),
            ),
            (
                "source_event_id",
                string(hex_bytes(source_event_id.as_bytes())),
            ),
        ])),
    )
}

fn cue_sort_key(cue: &AudioCueV1) -> (u64, ContentHash) {
    (cue.activation_simulation_tick, cue.cue_id)
}

fn emitter_key_value(key: AudioEmitterKeyV1) -> JcsValue {
    object([
        ("incarnation", number(key.incarnation)),
        ("subject_id", string(hex_bytes(key.subject_id.as_bytes()))),
    ])
}

fn optional_zone_value(zone: &Option<SchemaId>) -> JcsValue {
    string(
        zone.as_ref()
            .map_or_else(|| "none".to_owned(), |zone| zone.as_str().to_owned()),
    )
}

fn fact_value(fact: &AcousticFactV1) -> JcsValue {
    object([
        (
            "listener_id",
            string(hex_bytes(fact.listener_id.as_bytes())),
        ),
        ("loudness_class", string(fact.loudness_class.token())),
        (
            "occlusion_zone_or_none",
            optional_zone_value(&fact.occlusion_zone_or_none),
        ),
        ("source", emitter_key_value(fact.source)),
        ("tick", JcsValue::Number(fact.tick)),
    ])
}

#[cfg(test)]
mod tests {
    use super::{
        AcousticFactV1, AudioCueV1, AudioEmitterKeyV1, AudioEmitterRecordV1, AudioListenerRecordV1,
        AudioLoudnessClassV1, AudioPriorityClassV1, AudioSceneContractErrorV1,
        AudioSceneSnapshotV1,
    };
    use crate::ids::{AssetId, ContentHash, EventId, PersistentId, SchemaId};
    use crate::presentation::QuantizedPresentationTransformV1;
    use crate::project::{AssetRevisionRefV1, domain_hash};

    fn revision(seed: u8) -> AssetRevisionRefV1 {
        AssetRevisionRefV1 {
            asset_id: AssetId::from_bytes([seed; 16]),
            record_sha256: domain_hash("test.audio.clip", &[seed]),
        }
    }

    fn emitter_key(seed: u8) -> AudioEmitterKeyV1 {
        AudioEmitterKeyV1 {
            subject_id: PersistentId::from_bytes([seed; 16]),
            incarnation: 0,
        }
    }

    fn emitter(seed: u8) -> AudioEmitterRecordV1 {
        AudioEmitterRecordV1::new(
            emitter_key(seed),
            revision(seed),
            QuantizedPresentationTransformV1 {
                translation_micrometres: [i64::from(seed), 0, 0],
                ..QuantizedPresentationTransformV1::default()
            },
            AudioLoudnessClassV1::Normal,
            AudioPriorityClassV1::Normal,
            None,
            true,
        )
        .expect("emitter")
    }

    fn cue(seed: u8, tick: u64) -> AudioCueV1 {
        AudioCueV1::new(
            EventId::from_bytes([seed; 16]),
            0,
            tick,
            emitter_key(seed),
            revision(seed),
            AudioLoudnessClassV1::Loud,
            AudioPriorityClassV1::High,
            Some(SchemaId::new("nextengine.test.zone.cave").expect("zone")),
        )
        .expect("cue")
    }

    fn listener() -> AudioListenerRecordV1 {
        AudioListenerRecordV1::new(
            PersistentId::from_bytes([0x1a; 16]),
            QuantizedPresentationTransformV1::default(),
        )
        .expect("listener")
    }

    fn fact(seed: u8, tick: u64) -> AcousticFactV1 {
        AcousticFactV1 {
            tick,
            source: emitter_key(seed),
            listener_id: PersistentId::from_bytes([0x1a; 16]),
            loudness_class: AudioLoudnessClassV1::Loud,
            occlusion_zone_or_none: None,
        }
    }

    fn epoch() -> ContentHash {
        domain_hash("test.audio.epoch", b"epoch")
    }

    #[test]
    fn snapshot_construction_is_canonical_and_order_independent() {
        let forward = AudioSceneSnapshotV1::new(
            epoch(),
            7,
            42,
            listener(),
            vec![emitter(2), emitter(1)],
            vec![cue(3, 5), cue(1, 5), cue(2, 4)],
            vec![fact(3, 5), fact(1, 5)],
        )
        .expect("snapshot");
        let reverse = AudioSceneSnapshotV1::new(
            epoch(),
            7,
            42,
            listener(),
            vec![emitter(1), emitter(2)],
            vec![cue(2, 4), cue(1, 5), cue(3, 5)],
            vec![fact(1, 5), fact(3, 5)],
        )
        .expect("snapshot");
        assert_eq!(forward, reverse);
        forward.validate().expect("valid snapshot");
        assert_eq!(
            forward.emitters[0].emitter_key,
            emitter(1).emitter_key,
            "emitters sort by key"
        );
        assert_eq!(forward.cues[0].activation_simulation_tick, 4);
    }

    #[test]
    fn duplicate_emitter_key_and_cue_id_are_rejected() {
        let duplicate_emitter = emitter(1);
        assert_eq!(
            AudioSceneSnapshotV1::new(
                epoch(),
                0,
                0,
                listener(),
                vec![emitter(1), duplicate_emitter],
                Vec::new(),
                Vec::new(),
            ),
            Err(AudioSceneContractErrorV1::DuplicateEmitterKey)
        );
        let duplicate_cue = cue(1, 5);
        assert_eq!(
            AudioSceneSnapshotV1::new(
                epoch(),
                0,
                0,
                listener(),
                Vec::new(),
                vec![cue(1, 5), duplicate_cue],
                Vec::new(),
            ),
            Err(AudioSceneContractErrorV1::DuplicateCueId)
        );
    }

    #[test]
    fn cue_identity_is_derived_and_stable() {
        let first = cue(9, 12);
        let second = AudioCueV1::new(
            EventId::from_bytes([9; 16]),
            0,
            12,
            emitter_key(9),
            revision(9),
            AudioLoudnessClassV1::Loud,
            AudioPriorityClassV1::High,
            Some(SchemaId::new("nextengine.test.zone.cave").expect("zone")),
        )
        .expect("cue");
        assert_eq!(first.cue_id, second.cue_id);
        assert_eq!(first, second);
        let different_slot = AudioCueV1::new(
            EventId::from_bytes([9; 16]),
            1,
            12,
            emitter_key(9),
            revision(9),
            AudioLoudnessClassV1::Loud,
            AudioPriorityClassV1::High,
            Some(SchemaId::new("nextengine.test.zone.cave").expect("zone")),
        )
        .expect("cue");
        assert_ne!(first.cue_id, different_slot.cue_id);
        first.validate().expect("valid cue");
        let mut tampered = first.clone();
        tampered.source_cue_slot = 3;
        assert_eq!(
            tampered.validate(),
            Err(AudioSceneContractErrorV1::InvalidCueIdentity)
        );
    }

    #[test]
    fn zero_clip_hash_and_invalid_orientation_are_rejected() {
        assert_eq!(
            AudioEmitterRecordV1::new(
                emitter_key(1),
                AssetRevisionRefV1 {
                    asset_id: AssetId::from_bytes([1; 16]),
                    record_sha256: ContentHash::default(),
                },
                QuantizedPresentationTransformV1::default(),
                AudioLoudnessClassV1::Normal,
                AudioPriorityClassV1::Normal,
                None,
                false,
            ),
            Err(AudioSceneContractErrorV1::InvalidAssetRevision)
        );
        let bad_orientation = QuantizedPresentationTransformV1 {
            orientation_q30: [0, 0, 0, 0],
            ..QuantizedPresentationTransformV1::default()
        };
        assert_eq!(
            AudioListenerRecordV1::new(PersistentId::from_bytes([1; 16]), bad_orientation),
            Err(AudioSceneContractErrorV1::InvalidOrientation)
        );
    }

    #[test]
    fn snapshot_hash_binds_epoch_sequence_and_tick() {
        let base = AudioSceneSnapshotV1::new(
            epoch(),
            7,
            42,
            listener(),
            vec![emitter(1)],
            vec![cue(1, 5)],
            vec![fact(1, 5)],
        )
        .expect("snapshot");
        let other_epoch = AudioSceneSnapshotV1::new(
            domain_hash("test.audio.epoch", b"other"),
            7,
            42,
            listener(),
            vec![emitter(1)],
            vec![cue(1, 5)],
            vec![fact(1, 5)],
        )
        .expect("snapshot");
        assert_ne!(base.canonical_hash, other_epoch.canonical_hash);
        let mut tampered = base.clone();
        tampered.simulation_tick = 43;
        assert_eq!(
            tampered.validate(),
            Err(AudioSceneContractErrorV1::HashMismatch)
        );
    }

    #[test]
    fn limits_are_enforced() {
        let emitters = (0..=1_024_u16)
            .map(|index| {
                let mut bytes = [0_u8; 16];
                bytes[..2].copy_from_slice(&index.to_le_bytes());
                AudioEmitterRecordV1::new(
                    AudioEmitterKeyV1 {
                        subject_id: PersistentId::from_bytes(bytes),
                        incarnation: 0,
                    },
                    revision(1),
                    QuantizedPresentationTransformV1::default(),
                    AudioLoudnessClassV1::Quiet,
                    AudioPriorityClassV1::Low,
                    None,
                    false,
                )
                .expect("emitter")
            })
            .collect::<Vec<_>>();
        assert_eq!(
            AudioSceneSnapshotV1::new(epoch(), 0, 0, listener(), emitters, Vec::new(), Vec::new(),),
            Err(AudioSceneContractErrorV1::LimitExceeded)
        );
    }
}
