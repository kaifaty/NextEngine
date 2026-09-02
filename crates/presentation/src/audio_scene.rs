//! Deterministic audio scene extraction (SPEC-08 baseline audio).
//!
//! The extractor consumes committed `DomainEvent` records, exact physics
//! snapshot poses and engine-owned binding profiles, and publishes one
//! immutable `AudioSceneSnapshotV1` per extraction boundary. It never reads
//! device/mixer state and never writes to simulation: the same committed
//! inputs always produce the same listener, emitter, cue and acoustic-fact
//! sets, independent of extraction order or host audio capability.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::command::{DomainEventEnvelopeV2, EventPayload};
use next_contracts::ids::{ContentHash, PersistentId, SchemaId};
use next_contracts::physics::{PhysicsBodyIdV1, PhysicsCanonicalSnapshotV2};
use next_contracts::presentation::QuantizedPresentationTransformV1;
use next_contracts::presentation::audio_scene::{
    AcousticFactV1, AudioCueV1, AudioEmitterKeyV1, AudioEmitterRecordV1, AudioListenerRecordV1,
    AudioLoudnessClassV1, AudioPriorityClassV1, AudioSceneContractErrorV1, AudioSceneSnapshotV1,
};
use next_contracts::project::AssetRevisionRefV1;
use next_contracts::rpg::RpgEventV1;

/// Which subject a one-shot cue is attached to: the principal `PersistentId`
/// named by the source event payload, or the listener itself (UI/non-spatial
/// activations).
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AudioCueEmitterSubjectV1 {
    EventPrincipal = 0,
    Listener = 1,
}

/// Engine-owned binding from one committed event schema to the exact clip,
/// loudness/priority class and occlusion zone its one-shot cue activates.
/// `subtitle_text_id_or_none` marks speech-like cues: the localized text
/// renders as a subtitle while the cue is active (SPEC-08 voice-absent
/// subtitle fallback); effect clips carry none.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AudioEventCueBindingV1 {
    pub event_schema_id: SchemaId,
    pub clip_revision: AssetRevisionRefV1,
    pub loudness_class: AudioLoudnessClassV1,
    pub priority_class: AudioPriorityClassV1,
    pub occlusion_zone_or_none: Option<SchemaId>,
    pub emitter_subject: AudioCueEmitterSubjectV1,
    pub subtitle_text_id_or_none: Option<SchemaId>,
}

/// Engine-owned binding for one continuous scene emitter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AudioEmitterBindingV1 {
    pub emitter_key: AudioEmitterKeyV1,
    pub clip_revision: AssetRevisionRefV1,
    pub loudness_class: AudioLoudnessClassV1,
    pub priority_class: AudioPriorityClassV1,
    pub occlusion_zone_or_none: Option<SchemaId>,
    pub looped: bool,
    pub physics_body_id: Option<PhysicsBodyIdV1>,
    pub fallback_transform: QuantizedPresentationTransformV1,
}

/// Listener binding: the durable listener subject and its pose source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AudioListenerBindingV1 {
    pub listener_id: PersistentId,
    pub physics_body_id: Option<PhysicsBodyIdV1>,
    pub fallback_transform: QuantizedPresentationTransformV1,
}

/// Extracts one immutable audio scene for one committed simulation tick.
///
/// `events` MUST be the committed events of exactly that tick in their
/// canonical publication order. Cue slots are assigned as zero: V1 bindings
/// declare at most one clip per event schema. An event principal without a
/// resolvable physics pose yields a cue without a scene emitter record; the
/// mixer treats such cues as non-spatial. A binding for an event schema whose
/// payload names no principal subject is a configuration error and rejects.
#[allow(
    clippy::too_many_arguments,
    reason = "scene, listener, emitter, cue and event inputs meet at one atomic extraction boundary"
)]
pub fn extract_audio_scene(
    snapshot_epoch: ContentHash,
    snapshot_sequence: u64,
    simulation_tick: u64,
    listener: &AudioListenerBindingV1,
    emitter_bindings: &[AudioEmitterBindingV1],
    cue_bindings: &[AudioEventCueBindingV1],
    events: &[DomainEventEnvelopeV2],
    physics: &PhysicsCanonicalSnapshotV2,
) -> Result<AudioSceneSnapshotV1, AudioSceneExtractionErrorV1> {
    let cue_bindings = canonical_cue_bindings(cue_bindings)?;
    let mut emitters = Vec::with_capacity(emitter_bindings.len());
    let mut emitter_keys = BTreeMap::new();
    for binding in emitter_bindings {
        let record = AudioEmitterRecordV1::new(
            binding.emitter_key,
            binding.clip_revision,
            resolve_transform(binding.physics_body_id, binding.fallback_transform, physics),
            binding.loudness_class,
            binding.priority_class,
            binding.occlusion_zone_or_none.clone(),
            binding.looped,
        )?;
        if emitter_keys.insert(binding.emitter_key, ()).is_some() {
            return Err(AudioSceneExtractionErrorV1::DuplicateEmitterBinding);
        }
        emitters.push(record);
    }

    let listener_record = AudioListenerRecordV1::new(
        listener.listener_id,
        resolve_transform(
            listener.physics_body_id,
            listener.fallback_transform,
            physics,
        ),
    )?;

    let mut cues = Vec::new();
    let mut acoustic_facts = Vec::new();
    for event in events {
        let Some(binding) = cue_bindings
            .iter()
            .find(|binding| binding.event_schema_id == event.schema_id)
        else {
            continue;
        };
        let emitter_key = match binding.emitter_subject {
            AudioCueEmitterSubjectV1::Listener => AudioEmitterKeyV1 {
                subject_id: listener.listener_id,
                incarnation: 0,
            },
            AudioCueEmitterSubjectV1::EventPrincipal => {
                let principal = event_principal_subject(event)
                    .ok_or(AudioSceneExtractionErrorV1::CuePrincipalMissing)?;
                AudioEmitterKeyV1 {
                    subject_id: principal,
                    incarnation: 0,
                }
            }
        };
        let cue = AudioCueV1::new(
            event.event_id,
            event.schema_id.clone(),
            0,
            event.tick,
            emitter_key,
            binding.clip_revision,
            binding.loudness_class,
            binding.priority_class,
            binding.occlusion_zone_or_none.clone(),
        )?;
        acoustic_facts.push(AcousticFactV1 {
            tick: event.tick,
            source: emitter_key,
            listener_id: listener.listener_id,
            loudness_class: binding.loudness_class,
            occlusion_zone_or_none: binding.occlusion_zone_or_none.clone(),
        });
        cues.push(cue);
        if binding.emitter_subject == AudioCueEmitterSubjectV1::EventPrincipal
            && !emitter_keys.contains_key(&emitter_key)
        {
            let principal_body = PhysicsBodyIdV1 {
                subject_id: emitter_key.subject_id,
                body_slot: 0,
            };
            if let Some(body) = physics.sorted_body_states.get(&principal_body) {
                emitters.push(AudioEmitterRecordV1::new(
                    emitter_key,
                    binding.clip_revision,
                    QuantizedPresentationTransformV1 {
                        translation_micrometres: body.pose.translation_micrometres,
                        orientation_q30: body.pose.rotation_q1_30,
                    },
                    binding.loudness_class,
                    binding.priority_class,
                    binding.occlusion_zone_or_none.clone(),
                    false,
                )?);
                emitter_keys.insert(emitter_key, ());
            }
        }
    }

    Ok(AudioSceneSnapshotV1::new(
        snapshot_epoch,
        snapshot_sequence,
        simulation_tick,
        listener_record,
        emitters,
        cues,
        acoustic_facts,
    )?)
}

#[derive(Debug)]
#[non_exhaustive]
pub enum AudioSceneExtractionErrorV1 {
    Contract(AudioSceneContractErrorV1),
    DuplicateCueBinding,
    DuplicateEmitterBinding,
    CuePrincipalMissing,
}

impl Display for AudioSceneExtractionErrorV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Contract(error) => write!(formatter, "{error}"),
            Self::DuplicateCueBinding => {
                formatter.write_str("audio cue binding event schema is duplicated")
            }
            Self::DuplicateEmitterBinding => {
                formatter.write_str("audio emitter binding key is duplicated")
            }
            Self::CuePrincipalMissing => {
                formatter.write_str("audio cue binding event payload names no principal subject")
            }
        }
    }
}

impl Error for AudioSceneExtractionErrorV1 {}

impl From<AudioSceneContractErrorV1> for AudioSceneExtractionErrorV1 {
    fn from(error: AudioSceneContractErrorV1) -> Self {
        Self::Contract(error)
    }
}

fn canonical_cue_bindings(
    bindings: &[AudioEventCueBindingV1],
) -> Result<Vec<AudioEventCueBindingV1>, AudioSceneExtractionErrorV1> {
    let mut sorted = bindings.to_vec();
    sorted.sort_by(|left, right| {
        left.event_schema_id
            .as_str()
            .cmp(right.event_schema_id.as_str())
    });
    if sorted
        .windows(2)
        .any(|pair| pair[0].event_schema_id == pair[1].event_schema_id)
    {
        return Err(AudioSceneExtractionErrorV1::DuplicateCueBinding);
    }
    Ok(sorted)
}

fn resolve_transform(
    physics_body_id: Option<PhysicsBodyIdV1>,
    fallback: QuantizedPresentationTransformV1,
    physics: &PhysicsCanonicalSnapshotV2,
) -> QuantizedPresentationTransformV1 {
    physics_body_id
        .and_then(|body_id| physics.sorted_body_states.get(&body_id))
        .map_or(fallback, |body| QuantizedPresentationTransformV1 {
            translation_micrometres: body.pose.translation_micrometres,
            orientation_q30: body.pose.rotation_q1_30,
        })
}

fn event_principal_subject(event: &DomainEventEnvelopeV2) -> Option<PersistentId> {
    match &event.payload {
        EventPayload::CommandCommitted { .. } => None,
        EventPayload::Rpg(event) => match event {
            RpgEventV1::DialogueAdvanced { dialogue_id, .. } => Some(*dialogue_id),
            RpgEventV1::QuestTransitioned { quest_id, .. } => Some(*quest_id),
            RpgEventV1::RelationshipAdjusted {
                relationship_id, ..
            } => Some(*relationship_id),
            RpgEventV1::SkillProficiencySet { character_id, .. } => Some(*character_id),
            RpgEventV1::ItemTransferred { item_id, .. } => Some(*item_id),
            RpgEventV1::EquipmentAssigned { equipment_id, .. } => Some(*equipment_id),
            RpgEventV1::InteractiveObjectTransitioned { object_id, .. } => Some(*object_id),
            RpgEventV1::CharacterResourceAdjusted { character_id, .. } => Some(*character_id),
            RpgEventV1::CommitmentTransitioned { commitment_id, .. } => Some(*commitment_id),
            RpgEventV1::BodyConditionChanged { character_id, .. }
            | RpgEventV1::BodyTreatmentAdvanced { character_id, .. } => Some(*character_id),
        },
        EventPayload::Physical(event) => match event {
            next_contracts::physics::PhysicalEventV1::CapsuleStepApplied { body_id, .. } => {
                Some(*body_id)
            }
        },
        EventPayload::WorldRoutine(event) => Some(event.subject_id),
        EventPayload::WorldPopulation(event) => Some(match event {
            next_contracts::world_population::WorldPopulationChangedV1::TierTransitioned {
                subject_id,
                ..
            }
            | next_contracts::world_population::WorldPopulationChangedV1::AbstractTransferred {
                subject_id,
                ..
            } => *subject_id,
        }),
        EventPayload::WorldActivity(event) => Some(event.subject_id),
        EventPayload::AgentCognition(event) => Some(event.subject_id),
        EventPayload::WaterVolume(_) | EventPayload::WaterFlow(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use next_contracts::command::{CommandPhase, DomainEventEnvelopeV2};
    use next_contracts::ids::{
        AssetId, CommandId, EventId, PersistentId, PhysicsWorldId, SchemaId,
    };
    use next_contracts::input::TickRateProfileV1;
    use next_contracts::physics::{
        AuthoritativeNumericProfileV1, PhysicsBodyIdV1, PhysicsCanonicalSnapshotV2,
        PhysicsCoordinateProfileV1, PhysicsLimitsProfileV1, PhysicsQuantizationProfileV1,
        PhysicsSolverSemanticsProfileV1, PhysicsWorldCatalogProfilesV1, PhysicsWorldCatalogV1,
    };
    use next_contracts::project::domain_hash;
    use next_contracts::rpg::RpgEventV1;

    use super::*;

    const INTERACTIVE_EVENT_SCHEMA: &str =
        next_contracts::rpg::RPG_EVENT_INTERACTIVE_OBJECT_TRANSITIONED_SCHEMA_ID;

    fn revision(seed: u8) -> AssetRevisionRefV1 {
        AssetRevisionRefV1 {
            asset_id: AssetId::from_bytes([seed; 16]),
            record_sha256: domain_hash("test.audio.clip", &[seed]),
        }
    }

    fn empty_physics() -> PhysicsCanonicalSnapshotV2 {
        let tick_rate = TickRateProfileV1::at_30_hz();
        let quantization = PhysicsQuantizationProfileV1::capsule_reference_v1().expect("q");
        let numeric =
            AuthoritativeNumericProfileV1::capsule_reference_v1(&quantization).expect("n");
        let catalog = PhysicsWorldCatalogV1::new(
            PhysicsWorldId::from_bytes([9; 16]),
            PhysicsWorldCatalogProfilesV1 {
                coordinate: PhysicsCoordinateProfileV1::reference_v1().expect("c"),
                limits: PhysicsLimitsProfileV1::reference_v1().expect("l"),
                solver: PhysicsSolverSemanticsProfileV1::grounded_capsule_v1().expect("s"),
                tick_rate_hash: tick_rate.profile_hash().expect("t"),
                authoritative_numeric_hash: numeric.profile_hash().expect("n"),
                quantization_hash: quantization.profile_hash().expect("q"),
            },
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::new(),
        )
        .expect("catalog");
        PhysicsCanonicalSnapshotV2::genesis(&catalog, &tick_rate, &numeric, &quantization)
            .expect("snapshot")
    }

    fn listener() -> AudioListenerBindingV1 {
        AudioListenerBindingV1 {
            listener_id: PersistentId::from_bytes([0x1a; 16]),
            physics_body_id: None,
            fallback_transform: QuantizedPresentationTransformV1::default(),
        }
    }

    fn interactive_event(object_seed: u8, tick: u64) -> DomainEventEnvelopeV2 {
        DomainEventEnvelopeV2::rpg(
            tick,
            CommandPhase::Outcome,
            CommandId::from_bytes([object_seed; 16]),
            0,
            RpgEventV1::InteractiveObjectTransitioned {
                object_id: PersistentId::from_bytes([object_seed; 16]),
                state_id: SchemaId::new("nextengine.test.state.activated").expect("state"),
            },
        )
        .expect("event")
    }

    fn cue_binding() -> AudioEventCueBindingV1 {
        AudioEventCueBindingV1 {
            event_schema_id: SchemaId::new(INTERACTIVE_EVENT_SCHEMA).expect("schema"),
            clip_revision: revision(0xa1),
            loudness_class: AudioLoudnessClassV1::Normal,
            priority_class: AudioPriorityClassV1::High,
            occlusion_zone_or_none: None,
            emitter_subject: AudioCueEmitterSubjectV1::EventPrincipal,
            subtitle_text_id_or_none: None,
        }
    }

    #[test]
    fn events_extract_cues_facts_and_snapshot_is_order_stable() {
        let events = vec![interactive_event(0x42, 7), interactive_event(0x43, 7)];
        let scene = extract_audio_scene(
            domain_hash("test.audio.epoch", b"epoch"),
            3,
            7,
            &listener(),
            &[],
            &[cue_binding()],
            &events,
            &empty_physics(),
        )
        .expect("scene");
        assert_eq!(scene.cues.len(), 2);
        assert_eq!(scene.acoustic_facts.len(), 2);
        // No physics body resolves for the principals, so no emitter records
        // are synthesized; cues stay non-spatial.
        assert!(scene.emitters.is_empty());
        // Cues are canonically sorted by (activation tick, cue ID), not by
        // input order; both principals are present exactly once.
        let mut subjects = scene
            .cues
            .iter()
            .map(|cue| cue.emitter_key.subject_id)
            .collect::<Vec<_>>();
        subjects.sort();
        assert_eq!(
            subjects,
            vec![
                PersistentId::from_bytes([0x42; 16]),
                PersistentId::from_bytes([0x43; 16]),
            ]
        );
        // Re-extraction from the same committed inputs is byte-exact.
        let repeated = extract_audio_scene(
            domain_hash("test.audio.epoch", b"epoch"),
            3,
            7,
            &listener(),
            &[],
            &[cue_binding()],
            &events,
            &empty_physics(),
        )
        .expect("scene");
        assert_eq!(scene, repeated);

        // Permuting identical input events cannot change the published scene:
        // snapshot construction applies the canonical cue order.
        let mut permuted = events.clone();
        permuted.reverse();
        let permuted_scene = extract_audio_scene(
            domain_hash("test.audio.epoch", b"epoch"),
            3,
            7,
            &listener(),
            &[],
            &[cue_binding()],
            &permuted,
            &empty_physics(),
        )
        .expect("scene");
        assert_eq!(scene, permuted_scene);
    }

    #[test]
    fn unbound_event_schemas_produce_no_cues() {
        let events = vec![interactive_event(0x42, 7)];
        let scene = extract_audio_scene(
            domain_hash("test.audio.epoch", b"epoch"),
            0,
            7,
            &listener(),
            &[],
            &[],
            &events,
            &empty_physics(),
        )
        .expect("scene");
        assert!(scene.cues.is_empty() && scene.acoustic_facts.is_empty());
    }

    #[test]
    fn listener_subject_binding_anchors_cue_to_listener() {
        let mut binding = cue_binding();
        binding.emitter_subject = AudioCueEmitterSubjectV1::Listener;
        let scene = extract_audio_scene(
            domain_hash("test.audio.epoch", b"epoch"),
            0,
            7,
            &listener(),
            &[],
            &[binding],
            &[interactive_event(0x42, 7)],
            &empty_physics(),
        )
        .expect("scene");
        assert_eq!(
            scene.cues[0].emitter_key.subject_id,
            PersistentId::from_bytes([0x1a; 16])
        );
    }

    #[test]
    fn command_committed_principal_binding_rejects() {
        let binding = AudioEventCueBindingV1 {
            event_schema_id: SchemaId::new("nextengine.event.command-committed").expect("schema"),
            ..cue_binding()
        };
        let event = DomainEventEnvelopeV2::command_committed(
            7,
            CommandPhase::Outcome,
            CommandId::from_bytes([1; 16]),
            0,
        )
        .expect("event");
        assert!(matches!(
            extract_audio_scene(
                domain_hash("test.audio.epoch", b"epoch"),
                0,
                7,
                &listener(),
                &[],
                &[binding],
                &[event],
                &empty_physics(),
            ),
            Err(AudioSceneExtractionErrorV1::CuePrincipalMissing)
        ));
    }

    #[test]
    fn duplicate_bindings_reject() {
        assert!(matches!(
            extract_audio_scene(
                domain_hash("test.audio.epoch", b"epoch"),
                0,
                7,
                &listener(),
                &[],
                &[cue_binding(), cue_binding()],
                &[],
                &empty_physics(),
            ),
            Err(AudioSceneExtractionErrorV1::DuplicateCueBinding)
        ));
        let emitter_binding = AudioEmitterBindingV1 {
            emitter_key: AudioEmitterKeyV1 {
                subject_id: PersistentId::from_bytes([0x42; 16]),
                incarnation: 0,
            },
            clip_revision: revision(0xa1),
            loudness_class: AudioLoudnessClassV1::Quiet,
            priority_class: AudioPriorityClassV1::Low,
            occlusion_zone_or_none: None,
            looped: true,
            physics_body_id: None,
            fallback_transform: QuantizedPresentationTransformV1::default(),
        };
        assert!(matches!(
            extract_audio_scene(
                domain_hash("test.audio.epoch", b"epoch"),
                0,
                7,
                &listener(),
                &[emitter_binding.clone(), emitter_binding],
                &[],
                &[],
                &empty_physics(),
            ),
            Err(AudioSceneExtractionErrorV1::DuplicateEmitterBinding)
        ));
    }

    #[test]
    fn declared_emitter_uses_fallback_transform_without_physics_body() {
        let binding = AudioEmitterBindingV1 {
            emitter_key: AudioEmitterKeyV1 {
                subject_id: PersistentId::from_bytes([0x42; 16]),
                incarnation: 0,
            },
            clip_revision: revision(0xa1),
            loudness_class: AudioLoudnessClassV1::Quiet,
            priority_class: AudioPriorityClassV1::Low,
            occlusion_zone_or_none: Some(SchemaId::new("nextengine.test.zone.cave").expect("z")),
            looped: true,
            physics_body_id: Some(PhysicsBodyIdV1 {
                subject_id: PersistentId::from_bytes([0x42; 16]),
                body_slot: 0,
            }),
            fallback_transform: QuantizedPresentationTransformV1 {
                translation_micrometres: [1_000_000, 0, 0],
                ..QuantizedPresentationTransformV1::default()
            },
        };
        let scene = extract_audio_scene(
            domain_hash("test.audio.epoch", b"epoch"),
            0,
            7,
            &listener(),
            &[binding],
            &[],
            &[],
            &empty_physics(),
        )
        .expect("scene");
        assert_eq!(scene.emitters.len(), 1);
        assert_eq!(
            scene.emitters[0].transform.translation_micrometres,
            [1_000_000, 0, 0]
        );
        scene.validate().expect("valid scene");
    }

    #[test]
    fn event_id_and_command_identity_drive_cue_identity() {
        let event = interactive_event(0x42, 7);
        let scene = extract_audio_scene(
            domain_hash("test.audio.epoch", b"epoch"),
            0,
            7,
            &listener(),
            &[],
            &[cue_binding()],
            std::slice::from_ref(&event),
            &empty_physics(),
        )
        .expect("scene");
        assert_eq!(scene.cues[0].source_event_id, event.event_id);
        assert_ne!(event.event_id, EventId::from_bytes([0x42; 16]));
    }
}
