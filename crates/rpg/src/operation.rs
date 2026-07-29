use std::collections::{BTreeMap, btree_map::Entry};

use next_contracts::{
    InventoryPayloadV1, PersistentId, RpgAggregateKindV1, RpgAggregatePayloadV1, RpgEventV1,
    RpgOperationPayloadV1, SkillProficiency, SkillProficiencyEntryV1,
};

use crate::{RpgAggregateKeyV1, RpgPlanBuildError, RpgState};

pub(super) fn apply_operation(
    state: &RpgState,
    staged_payloads: &mut BTreeMap<RpgAggregateKeyV1, RpgAggregatePayloadV1>,
    operation: &RpgOperationPayloadV1,
) -> Result<RpgEventV1, RpgPlanBuildError> {
    match operation {
        RpgOperationPayloadV1::AdvanceDialogue {
            dialogue_id,
            expected_node_id,
            next_node_id,
        } => {
            let key = RpgAggregateKeyV1::new(RpgAggregateKindV1::Dialogue, *dialogue_id);
            let payload = staged_payload_mut(state, staged_payloads, key)?;
            let RpgAggregatePayloadV1::Dialogue(dialogue) = payload else {
                return Err(RpgPlanBuildError::TransactionAborted);
            };
            if dialogue.node_id != *expected_node_id {
                return Err(RpgPlanBuildError::TransitionInvalid);
            }
            dialogue.node_id = next_node_id.clone();
            Ok(RpgEventV1::DialogueAdvanced {
                dialogue_id: *dialogue_id,
                node_id: next_node_id.clone(),
            })
        }
        RpgOperationPayloadV1::TransitionQuest {
            quest_id,
            expected_state_id,
            next_state_id,
        } => {
            let key = RpgAggregateKeyV1::new(RpgAggregateKindV1::Quest, *quest_id);
            let payload = staged_payload_mut(state, staged_payloads, key)?;
            let RpgAggregatePayloadV1::Quest(quest) = payload else {
                return Err(RpgPlanBuildError::TransactionAborted);
            };
            if quest.state_id != *expected_state_id {
                return Err(RpgPlanBuildError::TransitionInvalid);
            }
            quest.state_id = next_state_id.clone();
            Ok(RpgEventV1::QuestTransitioned {
                quest_id: *quest_id,
                state_id: next_state_id.clone(),
            })
        }
        RpgOperationPayloadV1::AdjustRelationship {
            relationship_id,
            dimension_id,
            delta,
        } => {
            let key = RpgAggregateKeyV1::new(RpgAggregateKindV1::Relationship, *relationship_id);
            let payload = staged_payload_mut(state, staged_payloads, key)?;
            let RpgAggregatePayloadV1::Relationship(relationship) = payload else {
                return Err(RpgPlanBuildError::TransactionAborted);
            };
            let dimension = relationship
                .dimensions
                .iter_mut()
                .find(|entry| entry.dimension_id == *dimension_id)
                .ok_or(RpgPlanBuildError::TransitionInvalid)?;
            dimension.value = dimension
                .value
                .checked_add(*delta)
                .ok_or(RpgPlanBuildError::TransitionInvalid)?;
            Ok(RpgEventV1::RelationshipAdjusted {
                relationship_id: *relationship_id,
                dimension_id: dimension_id.clone(),
                value: dimension.value,
            })
        }
        RpgOperationPayloadV1::SetSkillProficiency {
            character_id,
            skill_id,
            expected_value,
            new_value,
        } => {
            let key = RpgAggregateKeyV1::new(RpgAggregateKindV1::Character, *character_id);
            let payload = staged_payload_mut(state, staged_payloads, key)?;
            let RpgAggregatePayloadV1::Character(character) = payload else {
                return Err(RpgPlanBuildError::TransactionAborted);
            };
            let current = character
                .skills
                .iter()
                .find(|entry| entry.skill_id == *skill_id)
                .map_or(0, |entry| entry.proficiency.get());
            if current != *expected_value {
                return Err(RpgPlanBuildError::TransitionInvalid);
            }
            let proficiency = SkillProficiency::new(*new_value)
                .map_err(|_| RpgPlanBuildError::TransitionInvalid)?;
            match character
                .skills
                .iter_mut()
                .find(|entry| entry.skill_id == *skill_id)
            {
                Some(entry) => entry.proficiency = proficiency,
                None => character.skills.push(SkillProficiencyEntryV1 {
                    skill_id: skill_id.clone(),
                    proficiency,
                }),
            }
            character.skills.sort_unstable();
            Ok(RpgEventV1::SkillProficiencySet {
                character_id: *character_id,
                skill_id: skill_id.clone(),
                value: proficiency,
            })
        }
        RpgOperationPayloadV1::TransferItem {
            item_id,
            source_inventory_id,
            destination_inventory_id,
            quantity,
        } => {
            let item = state.aggregate(RpgAggregateKindV1::Item, *item_id).ok_or(
                RpgPlanBuildError::AggregateNotFound(RpgAggregateKindV1::Item),
            )?;
            let RpgAggregatePayloadV1::Item(item_payload) = &item.payload else {
                return Err(RpgPlanBuildError::TransactionAborted);
            };
            if item_payload.quantity != *quantity {
                return Err(RpgPlanBuildError::OwnershipConflict);
            }
            if let Some(source_inventory_id) = source_inventory_id {
                let key =
                    RpgAggregateKeyV1::new(RpgAggregateKindV1::Inventory, *source_inventory_id);
                let payload = staged_payload_mut(state, staged_payloads, key)?;
                let RpgAggregatePayloadV1::Inventory(inventory) = payload else {
                    return Err(RpgPlanBuildError::TransactionAborted);
                };
                ensure_not_reserved(inventory, *item_id)?;
                let index = inventory
                    .item_ids
                    .binary_search(item_id)
                    .map_err(|_| RpgPlanBuildError::OwnershipConflict)?;
                inventory.item_ids.remove(index);
            }
            if let Some(destination_inventory_id) = destination_inventory_id {
                let key = RpgAggregateKeyV1::new(
                    RpgAggregateKindV1::Inventory,
                    *destination_inventory_id,
                );
                let payload = staged_payload_mut(state, staged_payloads, key)?;
                let RpgAggregatePayloadV1::Inventory(inventory) = payload else {
                    return Err(RpgPlanBuildError::TransactionAborted);
                };
                ensure_not_reserved(inventory, *item_id)?;
                if usize::try_from(inventory.capacity)
                    .map_or(true, |capacity| inventory.item_ids.len() >= capacity)
                    || inventory.item_ids.binary_search(item_id).is_ok()
                {
                    return Err(RpgPlanBuildError::OwnershipConflict);
                }
                let index = inventory.item_ids.partition_point(|id| id < item_id);
                inventory.item_ids.insert(index, *item_id);
            }
            Ok(RpgEventV1::ItemTransferred {
                item_id: *item_id,
                source_inventory_id: *source_inventory_id,
                destination_inventory_id: *destination_inventory_id,
                quantity: *quantity,
            })
        }
        RpgOperationPayloadV1::AssignEquipment {
            equipment_id,
            inventory_id,
            item_id,
            slot_id,
        } => {
            let inventory = staged_or_original_payload(
                state,
                staged_payloads,
                RpgAggregateKeyV1::new(RpgAggregateKindV1::Inventory, *inventory_id),
            )?;
            let RpgAggregatePayloadV1::Inventory(inventory) = inventory else {
                return Err(RpgPlanBuildError::TransactionAborted);
            };
            if inventory.item_ids.binary_search(item_id).is_err() {
                return Err(RpgPlanBuildError::OwnershipConflict);
            }
            let key = RpgAggregateKeyV1::new(RpgAggregateKindV1::Equipment, *equipment_id);
            let payload = staged_payload_mut(state, staged_payloads, key)?;
            let RpgAggregatePayloadV1::Equipment(equipment) = payload else {
                return Err(RpgPlanBuildError::TransactionAborted);
            };
            match equipment
                .assignments
                .binary_search_by(|assignment| assignment.slot_id.cmp(slot_id))
            {
                Ok(_) => return Err(RpgPlanBuildError::OwnershipConflict),
                Err(index) => equipment.assignments.insert(
                    index,
                    next_contracts::EquipmentSlotAssignmentV1 {
                        slot_id: slot_id.clone(),
                        item_id: *item_id,
                    },
                ),
            }
            Ok(RpgEventV1::EquipmentAssigned {
                equipment_id: *equipment_id,
                item_id: *item_id,
                slot_id: slot_id.clone(),
            })
        }
        RpgOperationPayloadV1::TransitionInteractiveObject {
            object_id,
            expected_state_id,
            next_state_id,
        } => {
            let key = RpgAggregateKeyV1::new(RpgAggregateKindV1::InteractiveObject, *object_id);
            let payload = staged_payload_mut(state, staged_payloads, key)?;
            let RpgAggregatePayloadV1::InteractiveObject(object) = payload else {
                return Err(RpgPlanBuildError::TransactionAborted);
            };
            if object.state_id != *expected_state_id {
                return Err(RpgPlanBuildError::TransitionInvalid);
            }
            object.state_id = next_state_id.clone();
            Ok(RpgEventV1::InteractiveObjectTransitioned {
                object_id: *object_id,
                state_id: next_state_id.clone(),
            })
        }
        RpgOperationPayloadV1::AdjustCharacterResource {
            source_character_id: _,
            character_id,
            resource_id,
            expected_value,
            delta,
        } => {
            let key = RpgAggregateKeyV1::new(RpgAggregateKindV1::Character, *character_id);
            let payload = staged_payload_mut(state, staged_payloads, key)?;
            let RpgAggregatePayloadV1::Character(character) = payload else {
                return Err(RpgPlanBuildError::TransactionAborted);
            };
            let resource = character
                .resources
                .iter_mut()
                .find(|entry| entry.resource_id == *resource_id)
                .ok_or(RpgPlanBuildError::TransitionInvalid)?;
            if resource.current_value != *expected_value {
                return Err(RpgPlanBuildError::TransitionInvalid);
            }
            let next = resource
                .current_value
                .checked_add(*delta)
                .ok_or(RpgPlanBuildError::TransitionInvalid)?;
            if next < resource.minimum_value || next > resource.maximum_value {
                return Err(RpgPlanBuildError::TransitionInvalid);
            }
            resource.current_value = next;
            Ok(RpgEventV1::CharacterResourceAdjusted {
                character_id: *character_id,
                resource_id: resource_id.clone(),
                value: next,
            })
        }
    }
}

fn ensure_not_reserved(
    inventory: &InventoryPayloadV1,
    item_id: PersistentId,
) -> Result<(), RpgPlanBuildError> {
    if inventory
        .reservations
        .iter()
        .any(|reservation| reservation.item_id == item_id)
    {
        Err(RpgPlanBuildError::ReservationInvalid)
    } else {
        Ok(())
    }
}

fn staged_payload_mut<'a>(
    state: &RpgState,
    staged: &'a mut BTreeMap<RpgAggregateKeyV1, RpgAggregatePayloadV1>,
    key: RpgAggregateKeyV1,
) -> Result<&'a mut RpgAggregatePayloadV1, RpgPlanBuildError> {
    match staged.entry(key) {
        Entry::Occupied(entry) => Ok(entry.into_mut()),
        Entry::Vacant(entry) => {
            let payload = state
                .aggregates
                .get(&key)
                .ok_or(RpgPlanBuildError::AggregateNotFound(key.aggregate_kind))?
                .payload
                .clone();
            Ok(entry.insert(payload))
        }
    }
}

fn staged_or_original_payload<'a>(
    state: &'a RpgState,
    staged: &'a BTreeMap<RpgAggregateKeyV1, RpgAggregatePayloadV1>,
    key: RpgAggregateKeyV1,
) -> Result<&'a RpgAggregatePayloadV1, RpgPlanBuildError> {
    if let Some(payload) = staged.get(&key) {
        return Ok(payload);
    }
    state
        .aggregates
        .get(&key)
        .map(|aggregate| &aggregate.payload)
        .ok_or(RpgPlanBuildError::AggregateNotFound(key.aggregate_kind))
}
