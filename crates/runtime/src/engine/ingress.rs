use std::collections::{BTreeMap, BTreeSet};

use next_contracts::canonical::CanonicalDecodeLimits;
use next_contracts::command::WorldCommand;
use next_contracts::ids::{ContentHash, PersistentId, SchemaId};
use next_contracts::input::{
    CLOSED_INGRESS_BATCH_SCHEMA_VERSION, CORE_CAMERA_ORBIT_ACTION_ID, CORE_EQUIP_USE_ACTION_ID,
    CORE_INTERACT_ACTION_ID, CORE_MELEE_ACTION_ID, CORE_MOVE_ACTION_ID, CORE_PICKUP_ACTION_ID,
    CORE_UI_BACK_ACTION_ID, CORE_UI_CONFIRM_ACTION_ID, CORE_UI_NAVIGATE_ACTION_ID,
    ClosedIngressBatchBodyV1, ClosedIngressBatchV1, INPUT_MAPPING_RECEIPT_SCHEMA_VERSION,
    IngressAssignmentV1, IngressCheckpointV1, IngressEquivalenceReceiptV1,
    InputActionMappingResultV2, InputDerivedCommandRefV2, InputMappingCodeV1,
    InputMappingReceiptV1, InputMappingReceiptV2, InputSampleV1, PLAYER_ACTION_FRAME_SCHEMA_ID,
    PLAYER_ACTION_FRAME_SCHEMA_VERSION, PLAYER_ACTION_SOURCE_CLASS, PlayerActionFrameV1,
    PlayerActionPhaseV1, PlayerActionValueV1, PlayerControllerBindingV1,
    PlayerControllerRegistryV1, RuntimeAdmissionLimitsV1,
};
use next_contracts::physics::PhysicalCommandV1;

use super::error::RuntimeFatalError;
use super::order::sort_command_batch;
use super::pipeline::count;

pub(super) struct ClosedIngressExecution {
    pub(super) batch: ClosedIngressBatchV1,
    pub(super) mapping_receipts: Vec<InputMappingReceiptV1>,
    pub(super) mapping_receipts_v2: Vec<InputMappingReceiptV2>,
    pub(super) derived_commands: Vec<WorldCommand>,
    pub(super) pending_interactions: Vec<PendingInteractionIntent>,
    pub(super) deduplicated: u64,
}

struct PlayerActionMapping {
    receipts: Vec<InputMappingReceiptV1>,
    receipts_v2: Vec<InputMappingReceiptV2>,
    commands: Vec<WorldCommand>,
    pending_interactions: Vec<PendingInteractionIntent>,
}

#[derive(Clone, Debug)]
pub(super) struct PendingInteractionIntent {
    pub(super) assignment: IngressAssignmentV1,
    pub(super) controlled_body_id: PersistentId,
    pub(super) kind: InteractionIntentKind,
    pub(super) source_id: next_contracts::ids::InputSourceId,
    pub(super) source_sequence: u64,
    pub(super) payload_hash: ContentHash,
    pub(super) source_action_ordinal: u32,
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
        (SchemaId, next_contracts::ids::InputSourceId, u64),
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
        &assignments,
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
        mapping_receipts_v2: mapping.receipts_v2,
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
        &batch.body.input_assignments,
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
        mapping_receipts_v2: mapping.receipts_v2,
        derived_commands: mapping.commands,
        pending_interactions: mapping.pending_interactions,
        deduplicated: 0,
    })
}

fn map_player_actions(
    tick: u64,
    samples: &[InputSampleV1],
    assignments: &[IngressAssignmentV1],
    collision_keys: &BTreeSet<(String, next_contracts::ids::InputSourceId, u64)>,
    controllers: &PlayerControllerRegistryV1,
    interaction_enabled: bool,
) -> Result<PlayerActionMapping, RuntimeFatalError> {
    struct Candidate {
        binding: PlayerControllerBindingV1,
        direction_q15: Option<([i16; 2], u32)>,
        interaction: Option<(InteractionIntentKind, u32)>,
        assignment: IngressAssignmentV1,
        payload_hash: ContentHash,
        receipt_index: usize,
        sequence: u64,
    }

    let mut receipts = Vec::new();
    let mut receipts_v2 = Vec::new();
    let mut mapped: BTreeMap<PersistentId, Vec<Candidate>> = BTreeMap::new();
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
        receipts_v2.push(InputMappingReceiptV2 {
            schema_version: INPUT_MAPPING_RECEIPT_SCHEMA_VERSION,
            assigned_tick: tick,
            source_id: sample.source_id,
            source_sequence: sample.source_sequence,
            payload_hash,
            frame_code: code,
            action_results: action
                .as_ref()
                .map_or_else(Vec::new, |action| action.action_results.clone()),
            derived_commands: Vec::new(),
        });
        if let Some(action) = action {
            let assignment = assignments
                .iter()
                .find(|assignment| {
                    assignment.source_class == sample.source_class
                        && assignment.source_id == sample.source_id
                        && assignment.source_sequence == sample.source_sequence
                        && assignment.payload_hash == payload_hash
                })
                .cloned()
                .ok_or(RuntimeFatalError::IngressCheckpointCorrupt)?;
            mapped
                .entry(action.binding.controller_id)
                .or_default()
                .push(Candidate {
                    binding: action.binding.clone(),
                    direction_q15: action.direction_q15,
                    interaction: action.interaction,
                    assignment,
                    payload_hash,
                    receipt_index,
                    sequence: sample.source_sequence,
                });
        }
    }

    let mut commands = Vec::with_capacity(mapped.len());
    let mut pending_interactions = Vec::new();
    for candidates in mapped.into_values() {
        if candidates.len() != 1 {
            for candidate in candidates {
                receipts[candidate.receipt_index].code = InputMappingCodeV1::FrameInvalid;
                let receipt = &mut receipts_v2[candidate.receipt_index];
                receipt.frame_code = InputMappingCodeV1::FrameInvalid;
                for action in &mut receipt.action_results {
                    action.mapping_code = InputMappingCodeV1::FrameInvalid;
                }
            }
            continue;
        }
        let candidate = candidates
            .into_iter()
            .next()
            .expect("one candidate was checked above");
        if let Some((direction_q15, source_action_ordinal)) = candidate.direction_q15 {
            let command = WorldCommand::physical(
                candidate.binding.command_stream_id,
                candidate.binding.principal.clone(),
                candidate.sequence,
                tick,
                candidate.binding.controlled_body_id,
                PhysicalCommandV1::SetCapsuleLocomotionIntent { direction_q15 },
            )?;
            let command_id = command.compute_command_id()?;
            receipts[candidate.receipt_index].derived_command_id = Some(command_id);
            receipts_v2[candidate.receipt_index]
                .derived_commands
                .push(InputDerivedCommandRefV2 {
                    command_ordinal: 0,
                    source_action_ordinal,
                    mapper_command_slot: 0,
                    command_id,
                });
            commands.push(command);
        }
        if let Some((kind, source_action_ordinal)) = candidate.interaction {
            pending_interactions.push(PendingInteractionIntent {
                assignment: candidate.assignment,
                controlled_body_id: candidate.binding.controlled_body_id,
                kind,
                source_id: candidate.binding.source_id,
                source_sequence: candidate.sequence,
                payload_hash: candidate.payload_hash,
                source_action_ordinal,
            });
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
    receipts_v2.sort_by_key(|receipt| {
        (
            receipt.assigned_tick,
            receipt.source_id,
            receipt.source_sequence,
            receipt.payload_hash,
        )
    });
    Ok(PlayerActionMapping {
        receipts,
        receipts_v2,
        commands,
        pending_interactions,
    })
}

struct MappedPlayerFrame<'a> {
    binding: &'a PlayerControllerBindingV1,
    direction_q15: Option<([i16; 2], u32)>,
    interaction: Option<(InteractionIntentKind, u32)>,
    action_results: Vec<InputActionMappingResultV2>,
}

fn map_player_action_sample<'a>(
    sample: &InputSampleV1,
    controllers: &'a PlayerControllerRegistryV1,
    interaction_enabled: bool,
) -> Result<MappedPlayerFrame<'a>, InputMappingCodeV1> {
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
    // The runtime registry only ever holds validated bindings (bootstrap,
    // restore and activate_input_configuration all run binding.validate()),
    // so the immutable action-map/context-stack pair check is guaranteed.
    frame
        .validate_against_validated_binding(binding)
        .map_err(|_| InputMappingCodeV1::FrameInvalid)?;
    if frame.actions.is_empty() {
        return Err(InputMappingCodeV1::ActionUnmapped);
    }
    let mut direction_q15 = None;
    let mut interaction = None;
    for (frame_action_ordinal, action) in frame.actions.iter().enumerate() {
        let frame_action_ordinal =
            u32::try_from(frame_action_ordinal).map_err(|_| InputMappingCodeV1::FrameInvalid)?;
        match action.action_id.as_str() {
            CORE_MOVE_ACTION_ID => {
                direction_q15 = Some((
                    validate_movement_action(action.phase, action.value)?,
                    frame_action_ordinal,
                ));
            }
            CORE_CAMERA_ORBIT_ACTION_ID => {
                let _ = validate_vector_action(action.phase, action.value)?;
            }
            // Universal UI actions are validated and receipted here so the
            // canonical frame stays replayable evidence; their effects
            // (pause lifecycle request, menu/dialogue choice) are declared at
            // the host/session boundary and never become world commands.
            CORE_UI_NAVIGATE_ACTION_ID => {
                let _ = validate_vector_action(action.phase, action.value)?;
            }
            CORE_UI_CONFIRM_ACTION_ID | CORE_UI_BACK_ACTION_ID => {
                validate_ui_digital_action(action.phase, action.value)?;
            }
            CORE_INTERACT_ACTION_ID
            | CORE_PICKUP_ACTION_ID
            | CORE_EQUIP_USE_ACTION_ID
            | CORE_MELEE_ACTION_ID => {
                if !interaction_enabled {
                    return Err(InputMappingCodeV1::ActionUnmapped);
                }
                let kind = match action.action_id.as_str() {
                    CORE_INTERACT_ACTION_ID => InteractionIntentKind::General,
                    CORE_PICKUP_ACTION_ID => InteractionIntentKind::Pickup,
                    CORE_EQUIP_USE_ACTION_ID => InteractionIntentKind::EquipUse,
                    CORE_MELEE_ACTION_ID => InteractionIntentKind::Melee,
                    _ => unreachable!("matched interaction action identifier"),
                };
                match (action.phase, action.value) {
                    (PlayerActionPhaseV1::Started, PlayerActionValueV1::Digital(true)) => {
                        if interaction.is_some() {
                            return Err(InputMappingCodeV1::ActionUnmapped);
                        }
                        interaction = Some((kind, frame_action_ordinal));
                    }
                    (
                        PlayerActionPhaseV1::Completed | PlayerActionPhaseV1::Cancelled,
                        PlayerActionValueV1::Digital(false),
                    ) => {}
                    _ => return Err(InputMappingCodeV1::ValueOutOfProfile),
                }
            }
            _ => return Err(InputMappingCodeV1::ActionUnmapped),
        }
    }
    Ok(MappedPlayerFrame {
        binding,
        direction_q15,
        interaction,
        action_results: frame
            .actions
            .iter()
            .enumerate()
            .map(|(frame_action_ordinal, action)| {
                Ok(InputActionMappingResultV2 {
                    frame_action_ordinal: u32::try_from(frame_action_ordinal)
                        .map_err(|_| InputMappingCodeV1::FrameInvalid)?,
                    semantic_occurrence_ordinal: action.semantic_occurrence_ordinal,
                    action_id: action.action_id.clone(),
                    mapping_code: InputMappingCodeV1::Accepted,
                    first_command_ordinal: 0,
                    command_count: 0,
                })
            })
            .collect::<Result<Vec<_>, InputMappingCodeV1>>()?,
    })
}

pub(super) fn finalize_mapping_receipt_v2(
    receipt: &mut InputMappingReceiptV2,
) -> Result<(), RuntimeFatalError> {
    receipt.derived_commands.sort_by_key(|command| {
        (
            command.source_action_ordinal,
            command.mapper_command_slot,
            *command.command_id.as_bytes(),
        )
    });
    for (command_ordinal, command) in receipt.derived_commands.iter_mut().enumerate() {
        command.command_ordinal =
            u32::try_from(command_ordinal).map_err(|_| RuntimeFatalError::TraceCountExhausted)?;
    }
    let mut first_command_ordinal = 0_u32;
    for action in &mut receipt.action_results {
        action.first_command_ordinal = first_command_ordinal;
        action.command_count = u32::try_from(
            receipt
                .derived_commands
                .iter()
                .filter(|command| command.source_action_ordinal == action.frame_action_ordinal)
                .count(),
        )
        .map_err(|_| RuntimeFatalError::TraceCountExhausted)?;
        first_command_ordinal = first_command_ordinal
            .checked_add(action.command_count)
            .ok_or(RuntimeFatalError::TraceCountExhausted)?;
    }
    receipt.validate()?;
    Ok(())
}

fn validate_movement_action(
    phase: PlayerActionPhaseV1,
    value: PlayerActionValueV1,
) -> Result<[i16; 2], InputMappingCodeV1> {
    let value = validate_vector_action(phase, value)?;
    matches!(
        value,
        [0, 0] | [32_767, 0] | [-32_767, 0] | [0, 32_767] | [0, -32_767]
    )
    .then_some(value)
    .ok_or(InputMappingCodeV1::ValueOutOfProfile)
}

fn validate_vector_action(
    phase: PlayerActionPhaseV1,
    value: PlayerActionValueV1,
) -> Result<[i16; 2], InputMappingCodeV1> {
    let PlayerActionValueV1::Vector2Q15(value) = value else {
        return Err(InputMappingCodeV1::ValueOutOfProfile);
    };
    let valid = match phase {
        PlayerActionPhaseV1::Started | PlayerActionPhaseV1::Performed => {
            value.iter().all(|component| *component != i16::MIN)
        }
        PlayerActionPhaseV1::Completed | PlayerActionPhaseV1::Cancelled => value == [0, 0],
    };
    valid
        .then_some(value)
        .ok_or(InputMappingCodeV1::ValueOutOfProfile)
}

fn validate_ui_digital_action(
    phase: PlayerActionPhaseV1,
    value: PlayerActionValueV1,
) -> Result<(), InputMappingCodeV1> {
    match (phase, value) {
        (PlayerActionPhaseV1::Started, PlayerActionValueV1::Digital(true))
        | (
            PlayerActionPhaseV1::Completed | PlayerActionPhaseV1::Cancelled,
            PlayerActionValueV1::Digital(false),
        ) => Ok(()),
        _ => Err(InputMappingCodeV1::ValueOutOfProfile),
    }
}
