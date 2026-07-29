use std::collections::{BTreeMap, BTreeSet};

use next_contracts::{
    CLOSED_INGRESS_BATCH_SCHEMA_VERSION, CORE_EQUIP_USE_ACTION_ID, CORE_INTERACT_ACTION_ID,
    CORE_MELEE_ACTION_ID, CORE_MOVE_ACTION_ID, CORE_PICKUP_ACTION_ID, CanonicalDecodeLimits,
    ClosedIngressBatchBodyV1, ClosedIngressBatchV1, ContentHash, IngressAssignmentV1,
    IngressCheckpointV1, IngressEquivalenceReceiptV1, InputMappingCodeV1, InputMappingReceiptV1,
    InputSampleV1, PLAYER_ACTION_FRAME_SCHEMA_ID, PLAYER_ACTION_FRAME_SCHEMA_VERSION,
    PLAYER_ACTION_SOURCE_CLASS, PersistentId, PhysicalCommandV1, PlayerActionFrameV1,
    PlayerActionPhaseV1, PlayerActionValueV1, PlayerControllerBindingV1,
    PlayerControllerRegistryV1, RuntimeAdmissionLimitsV1, SchemaId, WorldCommand,
};

use super::{RuntimeFatalError, count, sort_command_batch};

pub(super) struct ClosedIngressExecution {
    pub(super) batch: ClosedIngressBatchV1,
    pub(super) mapping_receipts: Vec<InputMappingReceiptV1>,
    pub(super) derived_commands: Vec<WorldCommand>,
    pub(super) pending_interactions: Vec<PendingInteractionIntent>,
    pub(super) deduplicated: u64,
}

struct PlayerActionMapping {
    receipts: Vec<InputMappingReceiptV1>,
    commands: Vec<WorldCommand>,
    pending_interactions: Vec<PendingInteractionIntent>,
}

#[derive(Clone, Debug)]
pub(super) struct PendingInteractionIntent {
    pub(super) controlled_body_id: PersistentId,
    pub(super) kind: InteractionIntentKind,
    pub(super) source_id: next_contracts::InputSourceId,
    pub(super) source_sequence: u64,
    pub(super) payload_hash: ContentHash,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum InteractionIntentKind {
    General,
    Pickup,
    EquipUse,
    Melee,
}

pub(super) fn close_ingress(
    tick: u64,
    following_tick: u64,
    admission: &RuntimeAdmissionLimitsV1,
    controllers: &PlayerControllerRegistryV1,
    interaction_enabled: bool,
    checkpoint: &mut IngressCheckpointV1,
) -> Result<ClosedIngressExecution, RuntimeFatalError> {
    if checkpoint.current_tick != tick {
        return Err(RuntimeFatalError::IngressCheckpointCorrupt);
    }
    let next_generation = checkpoint
        .current_generation
        .checked_add(1)
        .ok_or(RuntimeFatalError::IngressGenerationExhausted)?;

    let mut samples = checkpoint.current_samples.clone();
    for sample in &mut samples {
        let bytes = sample.canonical_bytes()?;
        *sample = InputSampleV1::from_canonical_bytes(
            &bytes,
            CanonicalDecodeLimits::default(),
            admission,
        )?;
    }
    samples.sort_by(|left, right| {
        left.sort_key()
            .expect("validated input sample has a canonical sort key")
            .cmp(
                &right
                    .sort_key()
                    .expect("validated input sample has a canonical sort key"),
            )
    });
    let before_dedup = samples.len();
    samples.dedup_by(|left, right| {
        left.canonical_bytes()
            .expect("validated sample is canonical")
            == right
                .canonical_bytes()
                .expect("validated sample is canonical")
    });
    let deduplicated = count(
        before_dedup
            .checked_sub(samples.len())
            .ok_or(RuntimeFatalError::TraceCountExhausted)?,
    )?;

    let mut collisions: BTreeMap<
        (SchemaId, next_contracts::InputSourceId, u64),
        BTreeSet<ContentHash>,
    > = BTreeMap::new();
    for sample in &samples {
        collisions
            .entry((
                sample.source_class.clone(),
                sample.source_id,
                sample.source_sequence,
            ))
            .or_default()
            .insert(sample.payload_hash()?);
    }
    let collision_keys = collisions
        .iter()
        .filter_map(|(key, hashes)| {
            (hashes.len() > 1).then_some((key.0.as_str().to_owned(), key.1, key.2))
        })
        .collect::<BTreeSet<_>>();

    let mut assignments = Vec::new();
    let mut equivalence_receipts = Vec::new();
    for (key, hashes) in &collisions {
        if hashes.len() > 1 {
            equivalence_receipts.push(IngressEquivalenceReceiptV1::for_input_collision(
                tick,
                &key.0,
                key.1,
                key.2,
                hashes.clone(),
            )?);
        }
    }
    for sample in &samples {
        let key = (
            sample.source_class.as_str().to_owned(),
            sample.source_id,
            sample.source_sequence,
        );
        if !collision_keys.contains(&key) {
            assignments.push(IngressAssignmentV1::from_sample(
                checkpoint.current_generation,
                tick,
                sample,
            )?);
        }
    }
    assignments.sort();
    equivalence_receipts.sort();

    let mut mapping = map_player_actions(
        tick,
        &samples,
        &collision_keys,
        controllers,
        interaction_enabled,
    )?;
    sort_command_batch(&mut mapping.commands)?;
    let body = ClosedIngressBatchBodyV1 {
        schema_version: CLOSED_INGRESS_BATCH_SCHEMA_VERSION,
        queue_generation: checkpoint.current_generation,
        assigned_tick: tick,
        input_samples: samples,
        completion_signals: Vec::new(),
        input_assignments: assignments,
        completion_assignments: Vec::new(),
        equivalence_receipts,
    };
    let batch = ClosedIngressBatchV1::from_body(body)?;
    batch.validate(admission)?;

    checkpoint.current_tick = following_tick;
    checkpoint.current_generation = next_generation;
    checkpoint.current_samples = std::mem::take(&mut checkpoint.next_samples);
    checkpoint.last_closed_batch_hash = Some(batch.batch_hash);
    checkpoint.validate(admission)?;

    Ok(ClosedIngressExecution {
        batch,
        mapping_receipts: mapping.receipts,
        derived_commands: mapping.commands,
        pending_interactions: mapping.pending_interactions,
        deduplicated,
    })
}

pub(super) fn accept_closed_ingress(
    tick: u64,
    following_tick: u64,
    admission: &RuntimeAdmissionLimitsV1,
    controllers: &PlayerControllerRegistryV1,
    interaction_enabled: bool,
    checkpoint: &mut IngressCheckpointV1,
    batch: ClosedIngressBatchV1,
) -> Result<ClosedIngressExecution, RuntimeFatalError> {
    batch.validate(admission)?;
    if checkpoint.current_tick != tick
        || batch.body.assigned_tick != tick
        || batch.body.queue_generation != checkpoint.current_generation
    {
        return Err(RuntimeFatalError::IngressCheckpointCorrupt);
    }
    let next_generation = checkpoint
        .current_generation
        .checked_add(1)
        .ok_or(RuntimeFatalError::IngressGenerationExhausted)?;

    if !checkpoint.current_samples.is_empty() {
        let mut queued = checkpoint.current_samples.clone();
        queued.sort_by(|left, right| {
            left.sort_key()
                .expect("validated queued input has a sort key")
                .cmp(
                    &right
                        .sort_key()
                        .expect("validated queued input has a sort key"),
                )
        });
        queued.dedup_by(|left, right| {
            left.canonical_bytes().expect("validated queued input")
                == right.canonical_bytes().expect("validated queued input")
        });
        if queued != batch.body.input_samples {
            return Err(RuntimeFatalError::IngressCheckpointCorrupt);
        }
    }

    let assigned = batch
        .body
        .input_assignments
        .iter()
        .map(|assignment| {
            (
                assignment.source_class.as_str().to_owned(),
                assignment.source_id,
                assignment.source_sequence,
                assignment.payload_hash,
            )
        })
        .collect::<BTreeSet<_>>();
    let mut collision_keys = BTreeSet::new();
    for sample in &batch.body.input_samples {
        let key = (
            sample.source_class.as_str().to_owned(),
            sample.source_id,
            sample.source_sequence,
            sample.payload_hash()?,
        );
        if !assigned.contains(&key) {
            collision_keys.insert((key.0, key.1, key.2));
        }
    }
    if collision_keys.len() != batch.body.equivalence_receipts.len() {
        return Err(RuntimeFatalError::IngressCheckpointCorrupt);
    }
    let mut mapping = map_player_actions(
        tick,
        &batch.body.input_samples,
        &collision_keys,
        controllers,
        interaction_enabled,
    )?;
    sort_command_batch(&mut mapping.commands)?;

    checkpoint.current_tick = following_tick;
    checkpoint.current_generation = next_generation;
    checkpoint.current_samples = std::mem::take(&mut checkpoint.next_samples);
    checkpoint.last_closed_batch_hash = Some(batch.batch_hash);
    checkpoint.validate(admission)?;

    Ok(ClosedIngressExecution {
        batch,
        mapping_receipts: mapping.receipts,
        derived_commands: mapping.commands,
        pending_interactions: mapping.pending_interactions,
        deduplicated: 0,
    })
}

fn map_player_actions(
    tick: u64,
    samples: &[InputSampleV1],
    collision_keys: &BTreeSet<(String, next_contracts::InputSourceId, u64)>,
    controllers: &PlayerControllerRegistryV1,
    interaction_enabled: bool,
) -> Result<PlayerActionMapping, RuntimeFatalError> {
    enum MappedAction {
        Movement {
            binding: PlayerControllerBindingV1,
            sequence: u64,
            direction_q15: [i16; 2],
            receipt_index: usize,
        },
        Interaction {
            binding: PlayerControllerBindingV1,
            kind: InteractionIntentKind,
            sequence: u64,
            payload_hash: ContentHash,
        },
        Noop,
    }

    let mut receipts = Vec::new();
    let mut mapped = BTreeMap::new();
    for sample in samples {
        let key = (
            sample.source_class.as_str().to_owned(),
            sample.source_id,
            sample.source_sequence,
        );
        if collision_keys.contains(&key) {
            continue;
        }
        let payload_hash = sample.payload_hash()?;
        let mapped_action = map_player_action_sample(sample, controllers, interaction_enabled);
        let (code, action) = match mapped_action {
            Ok(action) => (InputMappingCodeV1::Accepted, Some(action)),
            Err(code) => (code, None),
        };
        let receipt_index = receipts.len();
        receipts.push(InputMappingReceiptV1 {
            assigned_tick: tick,
            source_id: sample.source_id,
            source_sequence: sample.source_sequence,
            payload_hash,
            code,
            derived_command_id: None,
        });
        if let Some(action) = action {
            let controller_id = match &action {
                MappedPlayerAction::Movement { binding, .. }
                | MappedPlayerAction::Interaction { binding, .. }
                | MappedPlayerAction::Noop { binding } => binding.controller_id,
            };
            let action = match action {
                MappedPlayerAction::Movement {
                    binding,
                    direction_q15,
                } => MappedAction::Movement {
                    binding: binding.clone(),
                    sequence: sample.source_sequence,
                    direction_q15,
                    receipt_index,
                },
                MappedPlayerAction::Interaction { binding, kind } => MappedAction::Interaction {
                    binding: binding.clone(),
                    kind,
                    sequence: sample.source_sequence,
                    payload_hash,
                },
                MappedPlayerAction::Noop { .. } => MappedAction::Noop,
            };
            mapped.insert(controller_id, action);
        }
    }

    let mut commands = Vec::with_capacity(mapped.len());
    let mut pending_interactions = Vec::new();
    for action in mapped.into_values() {
        match action {
            MappedAction::Movement {
                binding,
                sequence,
                direction_q15,
                receipt_index,
            } => {
                let command = WorldCommand::physical(
                    binding.command_stream_id,
                    binding.principal,
                    sequence,
                    tick,
                    binding.controlled_body_id,
                    PhysicalCommandV1::SetCapsuleLocomotionIntent { direction_q15 },
                )?;
                receipts[receipt_index].derived_command_id = Some(command.compute_command_id()?);
                commands.push(command);
            }
            MappedAction::Interaction {
                binding,
                kind,
                sequence,
                payload_hash,
            } => pending_interactions.push(PendingInteractionIntent {
                controlled_body_id: binding.controlled_body_id,
                kind,
                source_id: binding.source_id,
                source_sequence: sequence,
                payload_hash,
            }),
            MappedAction::Noop => {}
        }
    }
    receipts.sort_by_key(|receipt| {
        (
            receipt.assigned_tick,
            receipt.source_id,
            receipt.source_sequence,
            receipt.payload_hash,
        )
    });
    Ok(PlayerActionMapping {
        receipts,
        commands,
        pending_interactions,
    })
}

enum MappedPlayerAction<'a> {
    Movement {
        binding: &'a PlayerControllerBindingV1,
        direction_q15: [i16; 2],
    },
    Interaction {
        binding: &'a PlayerControllerBindingV1,
        kind: InteractionIntentKind,
    },
    Noop {
        binding: &'a PlayerControllerBindingV1,
    },
}

fn map_player_action_sample<'a>(
    sample: &InputSampleV1,
    controllers: &'a PlayerControllerRegistryV1,
    interaction_enabled: bool,
) -> Result<MappedPlayerAction<'a>, InputMappingCodeV1> {
    if sample.source_class.as_str() != PLAYER_ACTION_SOURCE_CLASS
        || sample.payload_schema_id.as_str() != PLAYER_ACTION_FRAME_SCHEMA_ID
        || sample.payload_schema_version != u32::from(PLAYER_ACTION_FRAME_SCHEMA_VERSION)
    {
        return Err(InputMappingCodeV1::PayloadSchemaUnsupported);
    }
    let Some(binding) = controllers.bindings.get(&sample.source_id) else {
        return Err(InputMappingCodeV1::PrincipalUnbound);
    };
    let frame = PlayerActionFrameV1::from_canonical_bytes(
        &sample.payload,
        CanonicalDecodeLimits::default(),
    )
    .map_err(|_| InputMappingCodeV1::FrameInvalid)?;
    if frame.controller_id != binding.controller_id
        || frame.logical_frame_sequence != sample.source_sequence
    {
        return Err(InputMappingCodeV1::FrameInvalid);
    }
    if frame.action_map_hash != binding.action_map_hash
        || frame.action_map_revision != binding.action_map_revision
    {
        return Err(InputMappingCodeV1::ActionMapStale);
    }
    if frame.context_stack_hash != binding.context_stack_hash
        || frame.context_stack_revision != binding.context_stack_revision
    {
        return Err(InputMappingCodeV1::ContextStackStale);
    }
    if frame.actions.is_empty() {
        return Err(InputMappingCodeV1::ActionUnmapped);
    }
    let has_movement = frame
        .actions
        .iter()
        .any(|action| action.action_id.as_str() == CORE_MOVE_ACTION_ID);
    let semantic_actions = frame
        .actions
        .iter()
        .filter_map(|action| match action.action_id.as_str() {
            CORE_INTERACT_ACTION_ID => Some(InteractionIntentKind::General),
            CORE_PICKUP_ACTION_ID => Some(InteractionIntentKind::Pickup),
            CORE_EQUIP_USE_ACTION_ID => Some(InteractionIntentKind::EquipUse),
            CORE_MELEE_ACTION_ID => Some(InteractionIntentKind::Melee),
            _ => None,
        })
        .collect::<Vec<_>>();
    let has_interaction = !semantic_actions.is_empty();
    let has_unknown = frame.actions.iter().any(|action| {
        !matches!(
            action.action_id.as_str(),
            CORE_MOVE_ACTION_ID
                | CORE_INTERACT_ACTION_ID
                | CORE_PICKUP_ACTION_ID
                | CORE_EQUIP_USE_ACTION_ID
                | CORE_MELEE_ACTION_ID
        )
    });
    if has_unknown
        || (has_movement && has_interaction)
        || semantic_actions.windows(2).any(|pair| pair[0] != pair[1])
    {
        return Err(InputMappingCodeV1::ActionUnmapped);
    }
    if has_interaction {
        if !interaction_enabled || frame.actions.len() != 1 {
            return Err(if interaction_enabled {
                InputMappingCodeV1::ValueOutOfProfile
            } else {
                InputMappingCodeV1::ActionUnmapped
            });
        }
        let action = &frame.actions[0];
        let kind = semantic_actions[0];
        return match (action.phase, action.value) {
            (PlayerActionPhaseV1::Started, PlayerActionValueV1::Digital(true)) => {
                Ok(MappedPlayerAction::Interaction { binding, kind })
            }
            (
                PlayerActionPhaseV1::Completed | PlayerActionPhaseV1::Cancelled,
                PlayerActionValueV1::Digital(false),
            ) => Ok(MappedPlayerAction::Noop { binding }),
            _ => Err(InputMappingCodeV1::ValueOutOfProfile),
        };
    }
    let mut direction = None;
    for action in &frame.actions {
        let PlayerActionValueV1::Vector2Q15(value) = action.value else {
            return Err(InputMappingCodeV1::ValueOutOfProfile);
        };
        let accepted = match action.phase {
            PlayerActionPhaseV1::Started | PlayerActionPhaseV1::Performed => {
                matches!(
                    value,
                    [32_767, 0] | [-32_767, 0] | [0, 32_767] | [0, -32_767]
                )
            }
            PlayerActionPhaseV1::Completed | PlayerActionPhaseV1::Cancelled => value == [0, 0],
        };
        if !accepted {
            return Err(InputMappingCodeV1::ValueOutOfProfile);
        }
        direction = Some(value);
    }
    Ok(MappedPlayerAction::Movement {
        binding,
        direction_q15: direction
            .expect("nonempty validated action frame has a final movement value"),
    })
}
