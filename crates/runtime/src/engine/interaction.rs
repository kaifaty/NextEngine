use std::collections::BTreeMap;

use next_contracts::canonical::CanonicalDecodeLimits;
use next_contracts::command::{CommandPayload, IssuerPrincipal, WorldCommand};
use next_contracts::identity::{CommandStreamRegistryV1, PrincipalRegistryV1};
use next_contracts::ids::{
    CapabilityId, CommandBodyHash, CommandStreamId, ContentHash, PersistentId, SchemaId, SystemId,
};
use next_contracts::input::{CORE_MELEE_ACTION_ID, PLAYER_INTERACTION_SYSTEM_ID};
use next_contracts::ledger::{
    CommandBodyArchiveV1, CommandFinalResultV1, CommandLedgerError, CommandLedgerV2,
    CommandReceiptSubjectV1,
};
use next_contracts::mechanics::{
    MechanicsContractError, RpgDefinitionRegistryV1, interaction_definition_hash,
};
use next_contracts::physics::{ClosedPhysicsContactBatchV1, ContactPhaseV1};
use next_contracts::rpg::{
    CORE_EQUIPMENT_MAIN_HAND_SLOT_ID, CORE_INTERACTIVE_OBJECT_ACTIVATED_STATE_ID,
    CORE_INTERACTIVE_OBJECT_COLLECTED_STATE_ID, CORE_INTERACTIVE_OBJECT_READY_STATE_ID,
    CoreDialogueQuestClosureError, RPG_COMMAND_CAPABILITY_ID,
};
use next_contracts::rpg::{
    RpgAggregateKindV1, RpgAggregateRefV1, RpgCommandV1, RpgOperationPayloadV1, RpgOperationV1,
    RpgPhysicalContactFactV1,
};
use next_mechanics::{AbilityInvocationV1, MechanicsHostError, compile_contact_ability_v1};
use next_physics_api::PhysicsWorldHost;
use next_rpg::RpgState;

use crate::authority::AuthorityRegistry;

use super::affordance::{InteractionAffordance, select_interaction_affordance};
use super::error::RuntimeFatalError;
use super::ingress::{InteractionIntentKind, PendingInteractionIntent};
use super::policy::core_rpg_policy_hash;
use super::targeting::ResolvedInteractionTargetingV1;

#[derive(Clone, Debug)]
pub(super) struct InteractionOutcomeRoute {
    system_id: SystemId,
    stream_id: CommandStreamId,
}

#[derive(Clone, Debug)]
pub(super) struct BuiltInInteractionOutcome {
    pub(super) proposal: crate::outcome::OutcomeProposal,
    pub(super) source_id: next_contracts::ids::InputSourceId,
    pub(super) source_sequence: u64,
    pub(super) payload_hash: ContentHash,
    pub(super) source_action_ordinal: u32,
}

#[derive(Clone, Debug)]
pub(super) struct BuiltInInteractionResolution {
    pub(super) outcomes: Vec<BuiltInInteractionOutcome>,
    pub(super) targeting: Vec<ResolvedInteractionTargetingV1>,
}

pub(super) fn rpg_physical_contact_facts(
    batch: &ClosedPhysicsContactBatchV1,
    physics_checkpoint_revision: u64,
) -> Result<Vec<RpgPhysicalContactFactV1>, RuntimeFatalError> {
    let mut facts = batch
        .events
        .iter()
        .filter(|event| matches!(event.phase, ContactPhaseV1::Begin | ContactPhaseV1::Persist))
        .filter_map(|event| {
            let first = event.participant_low.body_id.subject_id;
            let second = event.participant_high.body_id.subject_id;
            if first == second {
                return None;
            }
            let (subject_low, subject_high) = if first < second {
                (first, second)
            } else {
                (second, first)
            };
            Some(RpgPhysicalContactFactV1 {
                gameplay_tick: batch.gameplay_tick,
                contact_id: event.contact_id,
                subject_low,
                subject_high,
                physics_checkpoint_revision,
                source_snapshot_hash: event.source_snapshot_hash,
                contact_batch_hash: batch.batch_hash,
            })
        })
        .collect::<Vec<_>>();
    facts.sort_unstable();
    facts.dedup();
    for fact in &facts {
        fact.validate()
            .map_err(|_| RuntimeFatalError::PhysicalOutcomeInvariant)?;
    }
    Ok(facts)
}

pub(super) fn resolve_interaction_outcome_route(
    principals: &PrincipalRegistryV1,
    streams: &CommandStreamRegistryV1,
    authority: &AuthorityRegistry,
) -> Option<InteractionOutcomeRoute> {
    let system_id = SystemId::new(PLAYER_INTERACTION_SYSTEM_ID)
        .expect("built-in interaction system identifier is valid");
    let principal = IssuerPrincipal::InternalSystem(system_id.clone());
    let capability = CapabilityId::new(RPG_COMMAND_CAPABILITY_ID)
        .expect("built-in RPG capability identifier is valid");
    if !principals.is_active(&principal)
        || !authority.is_authenticated(&principal)
        || !authority
            .grants(&principal)
            .is_some_and(|grants| grants.contains(&capability))
    {
        return None;
    }
    let stream_id = streams
        .entries
        .iter()
        .find(|(key, _)| {
            key.principal == principal && key.stream_slot == 0 && key.stream_epoch == 0
        })
        .map(|(_, stream_id)| *stream_id)?;
    Some(InteractionOutcomeRoute {
        system_id,
        stream_id,
    })
}

pub(super) fn build_interaction_outcomes(
    intents: &[PendingInteractionIntent],
    context: InteractionBuildContext<'_>,
) -> Result<BuiltInInteractionResolution, RuntimeFatalError> {
    let InteractionBuildContext {
        route,
        physics,
        rpg,
        rpg_definitions,
        ledger,
        archive,
        archive_additions,
        gameplay_tick,
        physical_contact_facts,
        authoritative_revision,
    } = context;
    let Some(route) = route else {
        if intents.is_empty() {
            return Ok(BuiltInInteractionResolution {
                outcomes: Vec::new(),
                targeting: Vec::new(),
            });
        }
        return Err(RuntimeFatalError::InternalIdentityCollision);
    };
    let ready_state = SchemaId::new(CORE_INTERACTIVE_OBJECT_READY_STATE_ID)
        .expect("built-in interactive-object state identifier is valid");
    let activated_state = SchemaId::new(CORE_INTERACTIVE_OBJECT_ACTIVATED_STATE_ID)
        .expect("built-in interactive-object state identifier is valid");
    let collected_state = SchemaId::new(CORE_INTERACTIVE_OBJECT_COLLECTED_STATE_ID)
        .expect("built-in interactive-object state identifier is valid");
    let main_hand_slot = SchemaId::new(CORE_EQUIPMENT_MAIN_HAND_SLOT_ID)
        .expect("built-in equipment slot identifier is valid");
    let mut outcomes = Vec::new();
    let mut targeting_facts = Vec::new();
    let interaction_stream =
        ledger
            .streams
            .get(&route.stream_id)
            .ok_or(RuntimeFatalError::LedgerCorrupt(
                CommandLedgerError::StreamKeyMismatch,
            ))?;
    for (intent_index, intent) in intents.iter().enumerate() {
        // Input retries retain the original internal interaction sequence.
        // Once that sequence is finalized, re-resolving against newer world
        // state could select a different affordance and manufacture an
        // internal identity collision. Treat the finalized occurrence as the
        // accepted idempotent no-op that the ingress receipt describes.
        if interaction_stream
            .admission_high_watermark
            .is_some_and(|high_watermark| intent.source_sequence <= high_watermark)
        {
            continue;
        }
        let query_slot =
            u32::try_from(intent_index).map_err(|_| RuntimeFatalError::TraceCountExhausted)?;
        let payload = if intent.kind == InteractionIntentKind::Melee {
            let compiled = match compile_contact_ability_v1(
                rpg_definitions,
                &rpg.snapshot(),
                AbilityInvocationV1 {
                    gameplay_tick,
                    source_character_id: intent.controlled_body_id,
                    semantic_action_id: SchemaId::new(CORE_MELEE_ACTION_ID)
                        .expect("engine-owned melee action identifier is valid"),
                    prior_cooldown_commit_tick: None,
                    physical_contact_facts: physical_contact_facts.to_vec(),
                },
            ) {
                Ok(compiled) => compiled,
                Err(
                    MechanicsHostError::EquipmentRequired
                    | MechanicsHostError::ContactRequired
                    | MechanicsHostError::TargetInvalid
                    | MechanicsHostError::TargetResourceMissing
                    | MechanicsHostError::ResourceBounds
                    | MechanicsHostError::CooldownActive,
                ) => continue,
                Err(error) => return Err(RuntimeFatalError::Mechanics(error)),
            };
            let ability = rpg_definitions
                .abilities
                .iter()
                .find(|ability| {
                    next_contracts::mechanics::ability_definition_hash(ability)
                        == compiled.ability_definition_hash
                })
                .ok_or(RuntimeFatalError::InternalIdentityCollision)?;
            let prior_commit_tick = latest_ability_commit_tick(
                ledger,
                archive,
                archive_additions,
                intent.controlled_body_id,
                compiled.ability_definition_hash,
                gameplay_tick,
                ability.cooldown.duration_ticks,
            )?;
            match ability
                .cooldown
                .validate_boundary(prior_commit_tick, gameplay_tick)
            {
                Ok(()) => compiled.rpg_command,
                Err(MechanicsContractError::CooldownActive) => continue,
                Err(error) => {
                    return Err(RuntimeFatalError::Mechanics(MechanicsHostError::Contract(
                        error,
                    )));
                }
            }
        } else {
            let selection = select_interaction_affordance(
                intent,
                gameplay_tick,
                query_slot,
                route.stream_id,
                physics,
                rpg,
                rpg_definitions,
            )?;
            if let Some(targeting) = selection.targeting {
                targeting_facts.push(targeting);
            }
            let Some(affordance) = selection.affordance else {
                continue;
            };
            match affordance {
                InteractionAffordance::ActivateCoreSwitch {
                    object_id,
                    expected_revision,
                } => RpgCommandV1 {
                    operations: vec![RpgOperationV1 {
                        operation_slot: 0,
                        targets: vec![RpgAggregateRefV1 {
                            aggregate_kind: RpgAggregateKindV1::InteractiveObject,
                            persistent_id: object_id,
                            expected_revision,
                        }],
                        definition_policy_hashes: vec![core_rpg_policy_hash()],
                        payload: RpgOperationPayloadV1::TransitionInteractiveObject {
                            object_id,
                            expected_state_id: ready_state.clone(),
                            next_state_id: activated_state.clone(),
                        },
                    }],
                },
                InteractionAffordance::AdvanceDialogueQuest { binding } => {
                    let interaction = rpg_definitions
                        .interactions
                        .get(binding.interaction_definition_index)
                        .ok_or(RuntimeFatalError::CoreInteractionClosure(
                            CoreDialogueQuestClosureError,
                        ))?;
                    let dialogue_definition = rpg_definitions
                        .dialogue(interaction.dialogue_definition)
                        .ok_or(RuntimeFatalError::CoreInteractionClosure(
                            CoreDialogueQuestClosureError,
                        ))?;
                    let quest_definition = rpg_definitions
                        .quest(interaction.quest_definition)
                        .ok_or(RuntimeFatalError::CoreInteractionClosure(
                            CoreDialogueQuestClosureError,
                        ))?;
                    let relationship_definition = rpg_definitions
                        .relationship(interaction.relationship_definition)
                        .ok_or(RuntimeFatalError::CoreInteractionClosure(
                            CoreDialogueQuestClosureError,
                        ))?;
                    let dialogue_transition = dialogue_definition
                        .transitions
                        .iter()
                        .find(|transition| {
                            transition.transition_id == interaction.dialogue_transition_id
                        })
                        .ok_or(RuntimeFatalError::CoreInteractionClosure(
                            CoreDialogueQuestClosureError,
                        ))?;
                    let quest_transition = quest_definition
                        .transitions
                        .iter()
                        .find(|transition| {
                            transition.transition_id == interaction.quest_transition_id
                        })
                        .ok_or(RuntimeFatalError::CoreInteractionClosure(
                            CoreDialogueQuestClosureError,
                        ))?;
                    let policy = interaction_definition_hash(interaction);
                    RpgCommandV1 {
                        operations: vec![
                            RpgOperationV1 {
                                operation_slot: 0,
                                targets: vec![RpgAggregateRefV1 {
                                    aggregate_kind: RpgAggregateKindV1::Dialogue,
                                    persistent_id: binding.dialogue_id,
                                    expected_revision: binding.dialogue_revision,
                                }],
                                definition_policy_hashes: vec![policy],
                                payload: RpgOperationPayloadV1::AdvanceDialogue {
                                    dialogue_id: binding.dialogue_id,
                                    expected_node_id: dialogue_transition.source_state_id.clone(),
                                    next_node_id: dialogue_transition.target_state_id.clone(),
                                },
                            },
                            RpgOperationV1 {
                                operation_slot: 1,
                                targets: vec![RpgAggregateRefV1 {
                                    aggregate_kind: RpgAggregateKindV1::Quest,
                                    persistent_id: binding.quest_id,
                                    expected_revision: binding.quest_revision,
                                }],
                                definition_policy_hashes: vec![policy],
                                payload: RpgOperationPayloadV1::TransitionQuest {
                                    quest_id: binding.quest_id,
                                    expected_state_id: quest_transition.source_state_id.clone(),
                                    next_state_id: quest_transition.target_state_id.clone(),
                                },
                            },
                            RpgOperationV1 {
                                operation_slot: 2,
                                targets: vec![RpgAggregateRefV1 {
                                    aggregate_kind: RpgAggregateKindV1::Relationship,
                                    persistent_id: binding.relationship_id,
                                    expected_revision: binding.relationship_revision,
                                }],
                                definition_policy_hashes: vec![policy],
                                payload: RpgOperationPayloadV1::AdjustRelationship {
                                    relationship_id: binding.relationship_id,
                                    dimension_id: relationship_definition.dimension_id.clone(),
                                    delta: interaction.relationship_delta,
                                },
                            },
                        ],
                    }
                }
                InteractionAffordance::Pickup {
                    proxy_id,
                    proxy_revision,
                    item_id,
                    item_revision,
                    item_quantity,
                    destination_inventory_id,
                    destination_inventory_revision,
                } => RpgCommandV1 {
                    operations: vec![
                        RpgOperationV1 {
                            operation_slot: 0,
                            targets: vec![
                                RpgAggregateRefV1 {
                                    aggregate_kind: RpgAggregateKindV1::Item,
                                    persistent_id: item_id,
                                    expected_revision: item_revision,
                                },
                                RpgAggregateRefV1 {
                                    aggregate_kind: RpgAggregateKindV1::Inventory,
                                    persistent_id: destination_inventory_id,
                                    expected_revision: destination_inventory_revision,
                                },
                            ],
                            definition_policy_hashes: vec![core_rpg_policy_hash()],
                            payload: RpgOperationPayloadV1::TransferItem {
                                item_id,
                                source_inventory_id: None,
                                destination_inventory_id: Some(destination_inventory_id),
                                quantity: item_quantity,
                            },
                        },
                        RpgOperationV1 {
                            operation_slot: 1,
                            targets: vec![RpgAggregateRefV1 {
                                aggregate_kind: RpgAggregateKindV1::InteractiveObject,
                                persistent_id: proxy_id,
                                expected_revision: proxy_revision,
                            }],
                            definition_policy_hashes: vec![core_rpg_policy_hash()],
                            payload: RpgOperationPayloadV1::TransitionInteractiveObject {
                                object_id: proxy_id,
                                expected_state_id: ready_state.clone(),
                                next_state_id: collected_state.clone(),
                            },
                        },
                    ],
                },
                InteractionAffordance::EquipUse {
                    equipment_id,
                    equipment_revision,
                    inventory_id,
                    inventory_revision,
                    item_id,
                    item_revision,
                    slot_policy_hash,
                } => RpgCommandV1 {
                    operations: vec![RpgOperationV1 {
                        operation_slot: 0,
                        targets: vec![
                            RpgAggregateRefV1 {
                                aggregate_kind: RpgAggregateKindV1::Item,
                                persistent_id: item_id,
                                expected_revision: item_revision,
                            },
                            RpgAggregateRefV1 {
                                aggregate_kind: RpgAggregateKindV1::Inventory,
                                persistent_id: inventory_id,
                                expected_revision: inventory_revision,
                            },
                            RpgAggregateRefV1 {
                                aggregate_kind: RpgAggregateKindV1::Equipment,
                                persistent_id: equipment_id,
                                expected_revision: equipment_revision,
                            },
                        ],
                        definition_policy_hashes: vec![slot_policy_hash],
                        payload: RpgOperationPayloadV1::AssignEquipment {
                            equipment_id,
                            inventory_id,
                            item_id,
                            slot_id: main_hand_slot.clone(),
                        },
                    }],
                },
            }
        };
        let proposal = crate::outcome::OutcomeProposal::rpg(
            route.system_id.clone(),
            route.stream_id,
            intent.source_sequence,
            payload,
        )
        .with_precondition_revision(authoritative_revision);
        outcomes.push(BuiltInInteractionOutcome {
            proposal,
            source_id: intent.source_id,
            source_sequence: intent.source_sequence,
            payload_hash: intent.payload_hash,
            source_action_ordinal: intent.source_action_ordinal,
        });
    }
    Ok(BuiltInInteractionResolution {
        outcomes,
        targeting: targeting_facts,
    })
}

#[derive(Clone, Copy)]
pub(super) struct InteractionBuildContext<'a> {
    pub(super) route: Option<&'a InteractionOutcomeRoute>,
    pub(super) physics: &'a PhysicsWorldHost,
    pub(super) rpg: &'a RpgState,
    pub(super) rpg_definitions: &'a RpgDefinitionRegistryV1,
    pub(super) ledger: &'a CommandLedgerV2,
    pub(super) archive: &'a CommandBodyArchiveV1,
    pub(super) archive_additions: &'a BTreeMap<CommandBodyHash, std::sync::Arc<[u8]>>,
    pub(super) gameplay_tick: u64,
    pub(super) physical_contact_facts: &'a [RpgPhysicalContactFactV1],
    pub(super) authoritative_revision: u64,
}

fn latest_ability_commit_tick(
    ledger: &CommandLedgerV2,
    archive: &CommandBodyArchiveV1,
    archive_additions: &BTreeMap<CommandBodyHash, std::sync::Arc<[u8]>>,
    source_character_id: PersistentId,
    ability_definition_hash: ContentHash,
    gameplay_tick: u64,
    cooldown_duration_ticks: u32,
) -> Result<Option<u64>, RuntimeFatalError> {
    let mut finalized = BTreeMap::<CommandBodyHash, (&CommandFinalResultV1, u64)>::new();
    for stream in ledger.streams.values() {
        for receipt in stream.receipt_window.iter() {
            if let CommandReceiptSubjectV1::Command {
                canonical_body_ref, ..
            } = &receipt.subject
            {
                finalized.insert(
                    *canonical_body_ref,
                    (&receipt.result, receipt.finalized_at_tick),
                );
            }
        }
    }
    let cooldown_horizon = gameplay_tick.saturating_sub(u64::from(cooldown_duration_ticks));
    let mut latest = None;
    for (body_hash, bytes) in archive.entries().iter().chain(archive_additions) {
        let command = WorldCommand::from_canonical_bytes(bytes, CanonicalDecodeLimits::default())
            .map_err(CommandLedgerError::from)?;
        let CommandPayload::Rpg(rpg_command) = &command.payload else {
            continue;
        };
        let is_ability = rpg_command.operations.iter().any(|operation| {
            operation
                .definition_policy_hashes
                .binary_search(&ability_definition_hash)
                .is_ok()
                && matches!(
                    operation.payload,
                    RpgOperationPayloadV1::AdjustCharacterResource {
                        source_character_id: source,
                        ..
                    } if source == source_character_id
                )
        });
        if !is_ability || command.target_tick >= gameplay_tick {
            continue;
        }
        let candidate_tick = match finalized.get(body_hash) {
            Some((CommandFinalResultV1::Committed, finalized_at_tick)) => *finalized_at_tick,
            Some((
                CommandFinalResultV1::Rejected { .. } | CommandFinalResultV1::Collision { .. },
                _,
            )) => continue,
            None if command.target_tick >= cooldown_horizon => command.target_tick,
            None => continue,
        };
        latest = Some(latest.map_or(candidate_tick, |prior: u64| prior.max(candidate_tick)));
    }
    Ok(latest)
}
