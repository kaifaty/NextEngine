use next_contracts::ids::{CommandStreamId, ContentHash, PersistentId};
use next_contracts::mechanics::RpgDefinitionRegistryV1;
use next_contracts::physics::{PhysicsQueryHitV1, PhysicsQueryResultPayloadV1};
use next_contracts::project::AssetRevisionRefV1;
use next_contracts::rpg::{
    CORE_EQUIPMENT_MAIN_HAND_SLOT_ID, CORE_INTERACTIVE_OBJECT_READY_STATE_ID,
    CoreDialogueQuestClosureError,
};
use next_contracts::rpg::{DefinitionRefV1, RpgAggregateKindV1, RpgAggregatePayloadV1};
use next_physics_api::PhysicsWorldHost;
use next_rpg::RpgState;

use super::RuntimeFatalError;
use super::ingress::{InteractionIntentKind, PendingInteractionIntent};
use super::targeting::{ResolvedInteractionTargetingV1, resolve_interaction_targeting};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum InteractionAffordance {
    ActivateCoreSwitch {
        object_id: PersistentId,
        expected_revision: u64,
    },
    AdvanceDialogueQuest {
        binding: AuthoredDialogueQuestBindingV1,
    },
    Pickup {
        proxy_id: PersistentId,
        proxy_revision: u64,
        item_id: PersistentId,
        item_revision: u64,
        item_quantity: u32,
        destination_inventory_id: PersistentId,
        destination_inventory_revision: u64,
    },
    EquipUse {
        equipment_id: PersistentId,
        equipment_revision: u64,
        inventory_id: PersistentId,
        inventory_revision: u64,
        item_id: PersistentId,
        item_revision: u64,
        slot_policy_hash: ContentHash,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct AuthoredDialogueQuestBindingV1 {
    pub(super) npc_id: PersistentId,
    pub(super) player_id: PersistentId,
    pub(super) dialogue_id: PersistentId,
    pub(super) dialogue_revision: u64,
    pub(super) quest_id: PersistentId,
    pub(super) quest_revision: u64,
    pub(super) relationship_id: PersistentId,
    pub(super) relationship_revision: u64,
    pub(super) interaction_definition_index: usize,
    ready: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct InteractionCandidate {
    subject_id: PersistentId,
    target_hit: PhysicsQueryHitV1,
    affordance_tag: u8,
    dialogue_id: PersistentId,
    quest_id: PersistentId,
    affordance: InteractionAffordance,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct InteractionAffordanceSelection {
    pub(super) affordance: Option<InteractionAffordance>,
    pub(super) targeting: Option<ResolvedInteractionTargetingV1>,
}

pub(super) fn select_interaction_affordance(
    intent: &PendingInteractionIntent,
    gameplay_tick: u64,
    query_slot: u32,
    issuer_stream_id: CommandStreamId,
    physics: &PhysicsWorldHost,
    rpg: &RpgState,
    rpg_definitions: &RpgDefinitionRegistryV1,
) -> Result<InteractionAffordanceSelection, RuntimeFatalError> {
    let controlled_body_id = intent.controlled_body_id;
    let kind = intent.kind;
    if kind == InteractionIntentKind::EquipUse {
        return Ok(InteractionAffordanceSelection {
            affordance: select_equip_use_affordance(controlled_body_id, rpg),
            targeting: None,
        });
    }
    let targeting = resolve_interaction_targeting(
        intent,
        gameplay_tick,
        query_slot,
        issuer_stream_id,
        physics,
    )?;
    let PhysicsQueryResultPayloadV1::All {
        truncated: false,
        hits,
        ..
    } = &targeting.result.payload
    else {
        return Err(RuntimeFatalError::PhysicalOutcomeInvariant);
    };
    let dialogue_binding =
        resolve_dialogue_quest_binding_v2(rpg, rpg_definitions, controlled_body_id)
            .map_err(RuntimeFatalError::CoreInteractionClosure)?
            .filter(|binding| binding.ready);
    let mut candidates = Vec::new();
    for hit in hits {
        let target = hit.shape_id.body_id.subject_id;
        if rpg.interactive_object(target).is_some_and(|object| {
            object.state_id.as_str() == CORE_INTERACTIVE_OBJECT_READY_STATE_ID
                && match kind {
                    InteractionIntentKind::General => object.linked_item_id.is_none(),
                    InteractionIntentKind::Pickup => object.linked_item_id.is_some(),
                    InteractionIntentKind::EquipUse | InteractionIntentKind::Melee => false,
                }
        }) {
            let expected_revision = rpg
                .aggregate(RpgAggregateKindV1::InteractiveObject, target)
                .expect("typed object lookup closes aggregate lookup")
                .revision;
            let affordance = if let Some(item_id) = rpg
                .interactive_object(target)
                .and_then(|object| object.linked_item_id)
            {
                let Some(character) = rpg.character(controlled_body_id) else {
                    continue;
                };
                let Some(destination_inventory_id) = character.inventory_id else {
                    continue;
                };
                let Some(item) = rpg.aggregate(RpgAggregateKindV1::Item, item_id) else {
                    continue;
                };
                let Some(destination) =
                    rpg.aggregate(RpgAggregateKindV1::Inventory, destination_inventory_id)
                else {
                    continue;
                };
                let RpgAggregatePayloadV1::Item(item_payload) = &item.payload else {
                    continue;
                };
                InteractionAffordance::Pickup {
                    proxy_id: target,
                    proxy_revision: expected_revision,
                    item_id,
                    item_revision: item.revision,
                    item_quantity: item_payload.quantity,
                    destination_inventory_id,
                    destination_inventory_revision: destination.revision,
                }
            } else {
                InteractionAffordance::ActivateCoreSwitch {
                    object_id: target,
                    expected_revision,
                }
            };
            candidates.push(InteractionCandidate {
                subject_id: target,
                target_hit: hit.clone(),
                affordance_tag: 0,
                dialogue_id: PersistentId::from_bytes([0; 16]),
                quest_id: PersistentId::from_bytes([0; 16]),
                affordance,
            });
        }
        if kind == InteractionIntentKind::General
            && let Some(binding) = dialogue_binding.filter(|binding| binding.npc_id == target)
        {
            candidates.push(InteractionCandidate {
                subject_id: target,
                target_hit: hit.clone(),
                affordance_tag: 1,
                dialogue_id: binding.dialogue_id,
                quest_id: binding.quest_id,
                affordance: InteractionAffordance::AdvanceDialogueQuest { binding },
            });
        }
    }
    candidates.sort_by(|left, right| {
        left.target_hit
            .cmp_canonical_for_kind(
                &right.target_hit,
                next_contracts::physics::PhysicsQueryKindV1::ClosestPoint,
            )
            .then_with(|| left.subject_id.cmp(&right.subject_id))
            .then_with(|| left.affordance_tag.cmp(&right.affordance_tag))
            .then_with(|| left.dialogue_id.cmp(&right.dialogue_id))
            .then_with(|| left.quest_id.cmp(&right.quest_id))
    });
    Ok(InteractionAffordanceSelection {
        affordance: candidates
            .first()
            .map(|candidate| candidate.affordance.clone()),
        targeting: Some(targeting),
    })
}

fn select_equip_use_affordance(
    controlled_character_id: PersistentId,
    rpg: &RpgState,
) -> Option<InteractionAffordance> {
    let character = rpg.character(controlled_character_id)?;
    let inventory_id = character.inventory_id?;
    let equipment_id = character.equipment_id?;
    let inventory = rpg.inventory(inventory_id)?;
    let equipment = rpg.equipment(equipment_id)?;
    if equipment
        .assignments
        .iter()
        .any(|assignment| assignment.slot_id.as_str() == CORE_EQUIPMENT_MAIN_HAND_SLOT_ID)
    {
        return None;
    }
    let item_id = inventory.item_ids.iter().copied().find(|item_id| {
        !equipment
            .assignments
            .iter()
            .any(|assignment| assignment.item_id == *item_id)
    })?;
    let DefinitionRefV1::Exact {
        content_hash: slot_policy_hash,
        ..
    } = &equipment.slot_policy
    else {
        return None;
    };
    Some(InteractionAffordance::EquipUse {
        equipment_id,
        equipment_revision: rpg
            .aggregate(RpgAggregateKindV1::Equipment, equipment_id)?
            .revision,
        inventory_id,
        inventory_revision: rpg
            .aggregate(RpgAggregateKindV1::Inventory, inventory_id)?
            .revision,
        item_id,
        item_revision: rpg.aggregate(RpgAggregateKindV1::Item, item_id)?.revision,
        slot_policy_hash: *slot_policy_hash,
    })
}

pub(super) fn resolve_dialogue_quest_binding_v2(
    rpg: &RpgState,
    definitions: &RpgDefinitionRegistryV1,
    controlled_character_id: PersistentId,
) -> Result<Option<AuthoredDialogueQuestBindingV1>, CoreDialogueQuestClosureError> {
    if definitions.interactions.is_empty() {
        return Ok(None);
    }
    let snapshot = rpg.snapshot();
    let mut candidates = Vec::new();
    for (interaction_index, interaction) in definitions.interactions.iter().enumerate() {
        let dialogue_definition = definitions
            .dialogue(interaction.dialogue_definition)
            .ok_or(CoreDialogueQuestClosureError)?;
        let transition = dialogue_definition
            .transitions
            .iter()
            .find(|transition| transition.transition_id == interaction.dialogue_transition_id)
            .ok_or(CoreDialogueQuestClosureError)?;
        for aggregate in &snapshot.aggregates {
            let RpgAggregatePayloadV1::Dialogue(payload) = &aggregate.payload else {
                continue;
            };
            if payload.listener_id == controlled_character_id
                && aggregate_definition_asset(aggregate) == Some(interaction.dialogue_definition)
                && (payload.node_id == transition.source_state_id
                    || payload.node_id == transition.target_state_id)
            {
                candidates.push((interaction_index, aggregate, payload));
            }
        }
    }
    if candidates.is_empty() {
        let has_authored_state = snapshot.aggregates.iter().any(|aggregate| {
            let Some(asset) = aggregate_definition_asset(aggregate) else {
                return false;
            };
            definitions.interactions.iter().any(|interaction| {
                asset == interaction.dialogue_definition
                    || asset == interaction.quest_definition
                    || asset == interaction.relationship_definition
            })
        });
        return if has_authored_state {
            Err(CoreDialogueQuestClosureError)
        } else {
            Ok(None)
        };
    }
    if rpg
        .aggregate(RpgAggregateKindV1::Character, controlled_character_id)
        .is_none()
    {
        return Err(CoreDialogueQuestClosureError);
    }
    let mut bindings = Vec::new();
    for (interaction_index, dialogue_envelope, dialogue) in candidates {
        if rpg
            .aggregate(RpgAggregateKindV1::Character, dialogue.speaker_id)
            .is_none()
        {
            return Err(CoreDialogueQuestClosureError);
        }
        let interaction = definitions
            .interactions
            .get(interaction_index)
            .ok_or(CoreDialogueQuestClosureError)?;
        let dialogue_definition = definitions
            .dialogue(interaction.dialogue_definition)
            .ok_or(CoreDialogueQuestClosureError)?;
        let dialogue_transition = dialogue_definition
            .transitions
            .iter()
            .find(|transition| transition.transition_id == interaction.dialogue_transition_id)
            .ok_or(CoreDialogueQuestClosureError)?;
        let quest_definition = definitions
            .quest(interaction.quest_definition)
            .ok_or(CoreDialogueQuestClosureError)?;
        let quest_transition = quest_definition
            .transitions
            .iter()
            .find(|transition| transition.transition_id == interaction.quest_transition_id)
            .ok_or(CoreDialogueQuestClosureError)?;
        let relationship_definition = definitions
            .relationship(interaction.relationship_definition)
            .ok_or(CoreDialogueQuestClosureError)?;
        let mut quests = snapshot.aggregates.iter().filter_map(|aggregate| {
            let RpgAggregatePayloadV1::Quest(payload) = &aggregate.payload else {
                return None;
            };
            (aggregate_definition_asset(aggregate) == Some(interaction.quest_definition)
                && (payload.state_id == quest_transition.source_state_id
                    || payload.state_id == quest_transition.target_state_id))
                .then_some((aggregate, payload))
        });
        let (quest_envelope, quest) = quests.next().ok_or(CoreDialogueQuestClosureError)?;
        if quests.next().is_some() {
            return Err(CoreDialogueQuestClosureError);
        }

        let mut relationships = snapshot.aggregates.iter().filter_map(|aggregate| {
            let RpgAggregatePayloadV1::Relationship(payload) = &aggregate.payload else {
                return None;
            };
            (aggregate_definition_asset(aggregate) == Some(interaction.relationship_definition)
                && payload.source_id == dialogue.speaker_id
                && payload.target_id == controlled_character_id)
                .then_some((aggregate, payload))
        });
        let (relationship_envelope, relationship) =
            relationships.next().ok_or(CoreDialogueQuestClosureError)?;
        if relationships.next().is_some() {
            return Err(CoreDialogueQuestClosureError);
        }
        let trust = relationship
            .dimensions
            .iter()
            .find(|dimension| dimension.dimension_id == relationship_definition.dimension_id)
            .ok_or(CoreDialogueQuestClosureError)?
            .value;
        let completed_relationship_value = interaction
            .relationship_source_value
            .checked_add(interaction.relationship_delta)
            .ok_or(CoreDialogueQuestClosureError)?;
        let ready = if dialogue.node_id == dialogue_transition.source_state_id
            && quest.state_id == quest_transition.source_state_id
            && trust == interaction.relationship_source_value
        {
            true
        } else if dialogue.node_id == dialogue_transition.target_state_id
            && quest.state_id == quest_transition.target_state_id
            && trust == completed_relationship_value
        {
            false
        } else {
            return Err(CoreDialogueQuestClosureError);
        };
        bindings.push(AuthoredDialogueQuestBindingV1 {
            npc_id: dialogue.speaker_id,
            player_id: controlled_character_id,
            dialogue_id: dialogue_envelope.persistent_id,
            dialogue_revision: dialogue_envelope.revision,
            quest_id: quest_envelope.persistent_id,
            quest_revision: quest_envelope.revision,
            relationship_id: relationship_envelope.persistent_id,
            relationship_revision: relationship_envelope.revision,
            interaction_definition_index: interaction_index,
            ready,
        });
    }
    let mut ready = bindings.iter().copied().filter(|binding| binding.ready);
    if let Some(binding) = ready.next() {
        return if ready.next().is_none() {
            Ok(Some(binding))
        } else {
            Err(CoreDialogueQuestClosureError)
        };
    }
    if bindings.len() == 1 {
        Ok(bindings.into_iter().next())
    } else {
        Err(CoreDialogueQuestClosureError)
    }
}

fn aggregate_definition_asset(
    aggregate: &next_contracts::rpg::RpgAggregateEnvelopeV1,
) -> Option<AssetRevisionRefV1> {
    match &aggregate.definition_ref {
        DefinitionRefV1::Exact {
            asset_id,
            content_hash,
        } => Some(AssetRevisionRefV1 {
            asset_id: *asset_id,
            record_sha256: *content_hash,
        }),
        DefinitionRefV1::None => None,
    }
}
