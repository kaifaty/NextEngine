#![forbid(unsafe_code)]

use std::collections::{BTreeMap, btree_map::Entry};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use next_contracts::{
    CORE_INTERACTIVE_OBJECT_COLLECTED_STATE_ID, CharacterPayloadV1, CommandBodyHash, CommandId,
    ContentHash, DefinitionRefV1, DialoguePayloadV1, EquipmentPayloadV1,
    FactionMembershipPayloadV1, InventoryPayloadV1, PersistentId, QuestPayloadV1,
    RelationshipPayloadV1, RpgAggregateEnvelopeV1, RpgAggregateKindV1, RpgAggregatePayloadV1,
    RpgAggregateRefV1, RpgCommandV1, RpgContractErrorV1, RpgEventDraftV1, RpgEventV1,
    RpgOperationPayloadV1, RpgPhysicalContactFactV1, RpgReadSetEntryV1, RpgSnapshotV2,
    RpgTransactionPlanV1, RpgWriteSetEntryV1, SchemaId, SkillProficiency, SkillProficiencyEntryV1,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RpgAggregateKeyV1 {
    pub aggregate_kind: RpgAggregateKindV1,
    pub persistent_id: PersistentId,
}

impl RpgAggregateKeyV1 {
    #[must_use]
    pub const fn new(aggregate_kind: RpgAggregateKindV1, persistent_id: PersistentId) -> Self {
        Self {
            aggregate_kind,
            persistent_id,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RpgState {
    aggregates: BTreeMap<RpgAggregateKeyV1, Arc<RpgAggregateEnvelopeV1>>,
}

impl RpgState {
    pub fn from_snapshot(snapshot: RpgSnapshotV2) -> Result<Self, RpgStateError> {
        snapshot.validate().map_err(RpgStateError::Contract)?;
        let mut aggregates = BTreeMap::new();
        for aggregate in snapshot.aggregates {
            let key = RpgAggregateKeyV1::new(aggregate.aggregate_kind, aggregate.persistent_id);
            if aggregates.insert(key, Arc::new(aggregate)).is_some() {
                return Err(RpgStateError::DuplicateAggregate(key));
            }
        }
        let state = Self { aggregates };
        state.validate_cross_references()?;
        Ok(state)
    }

    #[must_use]
    pub fn snapshot(&self) -> RpgSnapshotV2 {
        RpgSnapshotV2 {
            aggregates: self
                .aggregates
                .values()
                .map(|aggregate| aggregate.as_ref().clone())
                .collect(),
        }
    }

    #[must_use]
    pub fn aggregate(
        &self,
        aggregate_kind: RpgAggregateKindV1,
        persistent_id: PersistentId,
    ) -> Option<&RpgAggregateEnvelopeV1> {
        self.aggregates
            .get(&RpgAggregateKeyV1::new(aggregate_kind, persistent_id))
            .map(Arc::as_ref)
    }

    #[must_use]
    pub fn character(&self, id: PersistentId) -> Option<&CharacterPayloadV1> {
        match &self.aggregate(RpgAggregateKindV1::Character, id)?.payload {
            RpgAggregatePayloadV1::Character(payload) => Some(payload),
            _ => None,
        }
    }

    #[must_use]
    pub fn inventory(&self, id: PersistentId) -> Option<&InventoryPayloadV1> {
        match &self.aggregate(RpgAggregateKindV1::Inventory, id)?.payload {
            RpgAggregatePayloadV1::Inventory(payload) => Some(payload),
            _ => None,
        }
    }

    #[must_use]
    pub fn equipment(&self, id: PersistentId) -> Option<&EquipmentPayloadV1> {
        match &self.aggregate(RpgAggregateKindV1::Equipment, id)?.payload {
            RpgAggregatePayloadV1::Equipment(payload) => Some(payload),
            _ => None,
        }
    }

    #[must_use]
    pub fn quest(&self, id: PersistentId) -> Option<&QuestPayloadV1> {
        match &self.aggregate(RpgAggregateKindV1::Quest, id)?.payload {
            RpgAggregatePayloadV1::Quest(payload) => Some(payload),
            _ => None,
        }
    }

    #[must_use]
    pub fn dialogue(&self, id: PersistentId) -> Option<&DialoguePayloadV1> {
        match &self.aggregate(RpgAggregateKindV1::Dialogue, id)?.payload {
            RpgAggregatePayloadV1::Dialogue(payload) => Some(payload),
            _ => None,
        }
    }

    #[must_use]
    pub fn relationship(&self, id: PersistentId) -> Option<&RelationshipPayloadV1> {
        match &self
            .aggregate(RpgAggregateKindV1::Relationship, id)?
            .payload
        {
            RpgAggregatePayloadV1::Relationship(payload) => Some(payload),
            _ => None,
        }
    }

    #[must_use]
    pub fn interactive_object(
        &self,
        id: PersistentId,
    ) -> Option<&next_contracts::InteractiveObjectPayloadV1> {
        match &self
            .aggregate(RpgAggregateKindV1::InteractiveObject, id)?
            .payload
        {
            RpgAggregatePayloadV1::InteractiveObject(payload) => Some(payload),
            _ => None,
        }
    }

    fn validate_cross_references(&self) -> Result<(), RpgStateError> {
        let mut inventory_membership = BTreeMap::new();
        for (key, aggregate) in &self.aggregates {
            aggregate.validate().map_err(RpgStateError::Contract)?;
            match &aggregate.payload {
                RpgAggregatePayloadV1::Character(payload) => {
                    if let Some(inventory_id) = payload.inventory_id {
                        let inventory = self
                            .inventory(inventory_id)
                            .ok_or(RpgStateError::DanglingAggregateReference)?;
                        if inventory.owner_id != key.persistent_id {
                            return Err(RpgStateError::OwnerReferenceMismatch);
                        }
                    }
                    if let Some(equipment_id) = payload.equipment_id {
                        let equipment = self
                            .equipment(equipment_id)
                            .ok_or(RpgStateError::DanglingAggregateReference)?;
                        if equipment.character_id != key.persistent_id {
                            return Err(RpgStateError::OwnerReferenceMismatch);
                        }
                    }
                }
                RpgAggregatePayloadV1::Inventory(payload) => {
                    if self
                        .aggregate(RpgAggregateKindV1::Character, payload.owner_id)
                        .is_none()
                    {
                        return Err(RpgStateError::DanglingAggregateReference);
                    }
                    if usize::try_from(payload.capacity)
                        .map_or(true, |capacity| payload.item_ids.len() > capacity)
                    {
                        return Err(RpgStateError::InventoryCapacityExceeded);
                    }
                    for item_id in &payload.item_ids {
                        if self.aggregate(RpgAggregateKindV1::Item, *item_id).is_none() {
                            return Err(RpgStateError::DanglingAggregateReference);
                        }
                        if inventory_membership
                            .insert(*item_id, key.persistent_id)
                            .is_some()
                        {
                            return Err(RpgStateError::ItemHasMultipleInventories);
                        }
                    }
                }
                RpgAggregatePayloadV1::Equipment(payload) => {
                    let character = self
                        .character(payload.character_id)
                        .ok_or(RpgStateError::DanglingAggregateReference)?;
                    let inventory_id = character
                        .inventory_id
                        .ok_or(RpgStateError::OwnerReferenceMismatch)?;
                    let inventory = self
                        .inventory(inventory_id)
                        .ok_or(RpgStateError::DanglingAggregateReference)?;
                    for assignment in &payload.assignments {
                        if !inventory.item_ids.contains(&assignment.item_id) {
                            return Err(RpgStateError::EquippedItemNotInInventory);
                        }
                    }
                }
                RpgAggregatePayloadV1::Dialogue(payload) => {
                    ensure_kind_exists(self, RpgAggregateKindV1::Character, payload.speaker_id)?;
                    ensure_kind_exists(self, RpgAggregateKindV1::Character, payload.listener_id)?;
                }
                RpgAggregatePayloadV1::FactionMembership(payload) => {
                    validate_membership_references(self, payload)?;
                }
                RpgAggregatePayloadV1::Relationship(payload) => {
                    if !self.has_relationship_endpoint(payload.source_id)
                        || !self.has_relationship_endpoint(payload.target_id)
                    {
                        return Err(RpgStateError::DanglingAggregateReference);
                    }
                }
                RpgAggregatePayloadV1::DivineStanding(payload) => {
                    ensure_kind_exists(
                        self,
                        RpgAggregateKindV1::Character,
                        payload.subject_character_id,
                    )?;
                }
                RpgAggregatePayloadV1::Item(_)
                | RpgAggregatePayloadV1::Quest(_)
                | RpgAggregatePayloadV1::Faction(_)
                | RpgAggregatePayloadV1::InteractiveObject(_) => {}
            }
        }
        Ok(())
    }

    fn has_relationship_endpoint(&self, id: PersistentId) -> bool {
        [RpgAggregateKindV1::Character, RpgAggregateKindV1::Faction]
            .into_iter()
            .any(|kind| self.aggregate(kind, id).is_some())
    }
}

fn validate_membership_references(
    state: &RpgState,
    payload: &FactionMembershipPayloadV1,
) -> Result<(), RpgStateError> {
    ensure_kind_exists(state, RpgAggregateKindV1::Character, payload.character_id)?;
    ensure_kind_exists(state, RpgAggregateKindV1::Faction, payload.faction_id)
}

fn ensure_kind_exists(
    state: &RpgState,
    kind: RpgAggregateKindV1,
    id: PersistentId,
) -> Result<(), RpgStateError> {
    if state.aggregate(kind, id).is_some() {
        Ok(())
    } else {
        Err(RpgStateError::DanglingAggregateReference)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RpgPlanningContextV1<'a> {
    pub gameplay_tick: u64,
    pub causal_command_id: CommandId,
    pub canonical_command_body_hash: CommandBodyHash,
    pub project_composition_lock_hash: ContentHash,
    pub schema_registry_hash: ContentHash,
    pub budget_policy_hash: ContentHash,
    pub active_definition_policy_hashes: &'a [ContentHash],
    pub physical_contact_facts: &'a [RpgPhysicalContactFactV1],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuiltRpgTransactionPlanV1(RpgTransactionPlanV1);

impl BuiltRpgTransactionPlanV1 {
    #[must_use]
    pub fn as_contract(&self) -> &RpgTransactionPlanV1 {
        &self.0
    }
}

pub fn build_transaction_plan_v1(
    state: &RpgState,
    command: &RpgCommandV1,
    context: RpgPlanningContextV1<'_>,
) -> Result<BuiltRpgTransactionPlanV1, RpgPlanBuildError> {
    command.validate().map_err(RpgPlanBuildError::Contract)?;
    if !strictly_ordered_unique(context.active_definition_policy_hashes) {
        return Err(RpgPlanBuildError::DefinitionMismatch);
    }
    let validated_fact_hashes = validate_physical_preconditions(state, command, context)?;

    let mut reads = BTreeMap::new();
    let mut staged_payloads = BTreeMap::new();
    let mut events = Vec::new();

    for operation in &command.operations {
        if !is_subset(
            &operation.definition_policy_hashes,
            context.active_definition_policy_hashes,
        ) {
            return Err(RpgPlanBuildError::DefinitionMismatch);
        }
        validate_operation_policy(state, operation)?;
        for target in &operation.targets {
            let key = RpgAggregateKeyV1::new(target.aggregate_kind, target.persistent_id);
            let aggregate = state
                .aggregates
                .get(&key)
                .ok_or(RpgPlanBuildError::AggregateNotFound(target.aggregate_kind))?;
            if aggregate.revision != target.expected_revision {
                return Err(RpgPlanBuildError::RevisionStale {
                    aggregate_kind: target.aggregate_kind,
                    current_revision: aggregate.revision,
                });
            }
            reads.entry(key).or_insert_with(|| aggregate.clone());
        }

        let event = apply_operation(state, &mut staged_payloads, &operation.payload)?;
        let (primary_aggregate_kind, primary_persistent_id) = event.primary_aggregate();
        events.push(RpgEventDraftV1 {
            operation_slot: operation.operation_slot,
            event_local_slot: 0,
            event_schema_id: SchemaId::new(event.schema_id())
                .map_err(|_| RpgPlanBuildError::TransactionAborted)?,
            primary_aggregate_kind,
            primary_persistent_id,
            event,
        });
    }

    let mut ordered_read_set = Vec::with_capacity(reads.len());
    for (key, aggregate) in &reads {
        ordered_read_set.push(RpgReadSetEntryV1 {
            aggregate_ref: RpgAggregateRefV1 {
                aggregate_kind: key.aggregate_kind,
                persistent_id: key.persistent_id,
                expected_revision: aggregate.revision,
            },
            state_hash: aggregate
                .state_hash()
                .map_err(|_| RpgPlanBuildError::TransactionAborted)?,
        });
    }

    let mut ordered_write_set = Vec::with_capacity(staged_payloads.len());
    for (key, payload) in staged_payloads {
        let before = reads
            .get(&key)
            .ok_or(RpgPlanBuildError::TransactionAborted)?;
        let after_revision = before
            .revision
            .checked_add(1)
            .ok_or(RpgPlanBuildError::RevisionExhausted)?;
        let after = RpgAggregateEnvelopeV1::new(
            before.persistent_id,
            before.schema_version,
            after_revision,
            before.definition_ref.clone(),
            before.provenance.clone(),
            payload,
        )
        .map_err(RpgPlanBuildError::Contract)?;
        ordered_write_set.push(RpgWriteSetEntryV1 {
            aggregate_kind: key.aggregate_kind,
            persistent_id: key.persistent_id,
            before_revision: before.revision,
            after_revision,
            before_state_hash: before
                .state_hash()
                .map_err(|_| RpgPlanBuildError::TransactionAborted)?,
            after_state_hash: after
                .state_hash()
                .map_err(|_| RpgPlanBuildError::TransactionAborted)?,
            after,
        });
    }

    let mut definition_policy_hashes = command
        .operations
        .iter()
        .flat_map(|operation| operation.definition_policy_hashes.iter().copied())
        .collect::<Vec<_>>();
    definition_policy_hashes.sort_unstable();
    definition_policy_hashes.dedup();

    let mut plan = RpgTransactionPlanV1 {
        causal_command_id: context.causal_command_id,
        canonical_command_body_hash: context.canonical_command_body_hash,
        project_composition_lock_hash: context.project_composition_lock_hash,
        schema_registry_hash: context.schema_registry_hash,
        definition_policy_hashes,
        validated_fact_hashes,
        ordered_operations: command.operations.clone(),
        ordered_read_set,
        ordered_write_set,
        ordered_event_drafts: events,
        budget_policy_hash: context.budget_policy_hash,
        plan_hash: ContentHash::from_bytes([0; 32]),
    };
    plan.plan_hash = plan
        .recompute_plan_hash()
        .map_err(|_| RpgPlanBuildError::TransactionAborted)?;
    plan.validate().map_err(RpgPlanBuildError::Contract)?;
    Ok(BuiltRpgTransactionPlanV1(plan))
}

fn validate_physical_preconditions(
    state: &RpgState,
    command: &RpgCommandV1,
    context: RpgPlanningContextV1<'_>,
) -> Result<Vec<ContentHash>, RpgPlanBuildError> {
    let mut validated_fact_hashes = Vec::new();
    for operation in &command.operations {
        let RpgOperationPayloadV1::TransferItem {
            item_id,
            source_inventory_id: None,
            destination_inventory_id: Some(destination_inventory_id),
            ..
        } = &operation.payload
        else {
            continue;
        };
        let inventory = state.inventory(*destination_inventory_id).ok_or(
            RpgPlanBuildError::AggregateNotFound(RpgAggregateKindV1::Inventory),
        )?;
        let mut matching_proxies = command.operations.iter().filter_map(|candidate| {
            let RpgOperationPayloadV1::TransitionInteractiveObject {
                object_id,
                next_state_id,
                ..
            } = &candidate.payload
            else {
                return None;
            };
            let object = state.interactive_object(*object_id)?;
            (object.linked_item_id == Some(*item_id)
                && next_state_id.as_str() == CORE_INTERACTIVE_OBJECT_COLLECTED_STATE_ID)
                .then_some(*object_id)
        });
        let proxy_id = matching_proxies
            .next()
            .ok_or(RpgPlanBuildError::PhysicalPreconditionMissing)?;
        if matching_proxies.next().is_some() {
            return Err(RpgPlanBuildError::PhysicalPreconditionMissing);
        }
        let fact_hash = context
            .physical_contact_facts
            .iter()
            .filter(|fact| {
                fact.gameplay_tick == context.gameplay_tick
                    && fact.connects(inventory.owner_id, proxy_id)
            })
            .map(|fact| {
                fact.validate().map_err(RpgPlanBuildError::Contract)?;
                fact.fact_hash()
                    .map_err(|_| RpgPlanBuildError::TransactionAborted)
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .min()
            .ok_or(RpgPlanBuildError::PhysicalPreconditionMissing)?;
        validated_fact_hashes.push(fact_hash);
    }
    validated_fact_hashes.sort_unstable();
    validated_fact_hashes.dedup();
    Ok(validated_fact_hashes)
}

fn validate_operation_policy(
    state: &RpgState,
    operation: &next_contracts::RpgOperationV1,
) -> Result<(), RpgPlanBuildError> {
    let RpgOperationPayloadV1::AssignEquipment { equipment_id, .. } = &operation.payload else {
        return Ok(());
    };
    let equipment = state
        .equipment(*equipment_id)
        .ok_or(RpgPlanBuildError::AggregateNotFound(
            RpgAggregateKindV1::Equipment,
        ))?;
    let DefinitionRefV1::Exact { content_hash, .. } = &equipment.slot_policy else {
        return Err(RpgPlanBuildError::DefinitionMismatch);
    };
    if operation
        .definition_policy_hashes
        .binary_search(content_hash)
        .is_err()
    {
        return Err(RpgPlanBuildError::DefinitionMismatch);
    }
    Ok(())
}

fn apply_operation(
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

pub fn recheck_transaction_plan_v1(
    current: &RpgState,
    plan: &BuiltRpgTransactionPlanV1,
) -> Result<(), RpgPlanMaterializeError> {
    plan.0
        .validate()
        .map_err(RpgPlanMaterializeError::Contract)?;
    for read in &plan.0.ordered_read_set {
        let aggregate = current
            .aggregate(
                read.aggregate_ref.aggregate_kind,
                read.aggregate_ref.persistent_id,
            )
            .ok_or(RpgPlanMaterializeError::PlanStale)?;
        if aggregate.revision != read.aggregate_ref.expected_revision
            || aggregate
                .state_hash()
                .map_err(|_| RpgPlanMaterializeError::TransactionAborted)?
                != read.state_hash
        {
            return Err(RpgPlanMaterializeError::PlanStale);
        }
    }
    Ok(())
}

pub fn materialize_transaction_plan_v1(
    current: &RpgState,
    plan: &BuiltRpgTransactionPlanV1,
) -> Result<RpgState, RpgPlanMaterializeError> {
    recheck_transaction_plan_v1(current, plan)?;
    let mut next = current.clone();
    for write in &plan.0.ordered_write_set {
        let key = RpgAggregateKeyV1::new(write.aggregate_kind, write.persistent_id);
        next.aggregates.insert(key, Arc::new(write.after.clone()));
    }
    next.validate_cross_references()
        .map_err(RpgPlanMaterializeError::State)?;
    Ok(next)
}

fn is_subset(needles: &[ContentHash], haystack: &[ContentHash]) -> bool {
    needles
        .iter()
        .all(|needle| haystack.binary_search(needle).is_ok())
}

fn strictly_ordered_unique<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RpgStateError {
    Contract(RpgContractErrorV1),
    DuplicateAggregate(RpgAggregateKeyV1),
    DanglingAggregateReference,
    OwnerReferenceMismatch,
    InventoryCapacityExceeded,
    ItemHasMultipleInventories,
    EquippedItemNotInInventory,
}

impl RpgStateError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::Contract(error) => error.stable_code(),
            Self::DuplicateAggregate(_) => "RPG_DUPLICATE_AGGREGATE",
            Self::DanglingAggregateReference => "RPG_DANGLING_AGGREGATE_REFERENCE",
            Self::OwnerReferenceMismatch => "RPG_OWNERSHIP_CONFLICT",
            Self::InventoryCapacityExceeded => "RPG_INVENTORY_CAPACITY_EXCEEDED",
            Self::ItemHasMultipleInventories => "RPG_OWNERSHIP_CONFLICT",
            Self::EquippedItemNotInInventory => "RPG_OWNERSHIP_CONFLICT",
        }
    }
}

impl Display for RpgStateError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for RpgStateError {}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RpgPlanBuildError {
    Contract(RpgContractErrorV1),
    AggregateNotFound(RpgAggregateKindV1),
    RevisionStale {
        aggregate_kind: RpgAggregateKindV1,
        current_revision: u64,
    },
    RevisionExhausted,
    TransitionInvalid,
    OwnershipConflict,
    ReservationInvalid,
    PhysicalPreconditionMissing,
    DefinitionMismatch,
    CommitmentRejected,
    TransactionAborted,
}

impl RpgPlanBuildError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::Contract(error) => error.stable_code(),
            Self::AggregateNotFound(_) => "RPG_AGGREGATE_NOT_FOUND",
            Self::RevisionStale { .. } => "RPG_REVISION_STALE",
            Self::RevisionExhausted => "RPG_REVISION_EXHAUSTED",
            Self::TransitionInvalid => "RPG_TRANSITION_INVALID",
            Self::OwnershipConflict => "RPG_OWNERSHIP_CONFLICT",
            Self::ReservationInvalid => "RPG_RESERVATION_INVALID",
            Self::PhysicalPreconditionMissing => "RPG_PHYSICAL_PRECONDITION_MISSING",
            Self::DefinitionMismatch => "RPG_DEFINITION_MISMATCH",
            Self::CommitmentRejected => "RPG_COMMITMENT_REJECTED",
            Self::TransactionAborted => "RPG_TRANSACTION_ABORTED",
        }
    }
}

impl Display for RpgPlanBuildError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RevisionStale {
                aggregate_kind,
                current_revision,
            } => write!(
                formatter,
                "{}: {:?}@{}",
                self.stable_code(),
                aggregate_kind,
                current_revision
            ),
            Self::AggregateNotFound(kind) => {
                write!(formatter, "{}: {kind:?}", self.stable_code())
            }
            _ => formatter.write_str(self.stable_code()),
        }
    }
}

impl Error for RpgPlanBuildError {}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RpgPlanMaterializeError {
    Contract(RpgContractErrorV1),
    State(RpgStateError),
    PlanStale,
    TransactionAborted,
}

impl RpgPlanMaterializeError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::Contract(error) => error.stable_code(),
            Self::State(error) => error.stable_code(),
            Self::PlanStale => "RPG_PLAN_STALE",
            Self::TransactionAborted => "RPG_TRANSACTION_ABORTED",
        }
    }
}

impl Display for RpgPlanMaterializeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for RpgPlanMaterializeError {}

#[cfg(test)]
mod tests {
    use next_contracts::{
        AssetId, CORE_INTERACTIVE_OBJECT_COLLECTED_STATE_ID,
        CORE_INTERACTIVE_OBJECT_READY_STATE_ID, CharacterPayloadV1, CommandBodyHash, CommandId,
        ContentHash, DefinitionRefV1, DialoguePayloadV1, EquipmentPayloadV1,
        InteractiveObjectPayloadV1, InventoryPayloadV1, ItemPayloadV1, PhysicsContactId,
        ProvenanceBindingV1, QuestPayloadV1, RelationshipDimensionV1, RelationshipPayloadV1,
        RpgAggregateEnvelopeV1, RpgAggregateKindV1, RpgAggregatePayloadV1, RpgAggregateRefV1,
        RpgCommandV1, RpgOperationPayloadV1, RpgOperationV1, RpgPhysicalContactFactV1,
        RpgSnapshotV2, SchemaId,
    };

    use super::{
        RpgPlanBuildError, RpgPlanMaterializeError, RpgPlanningContextV1, RpgState,
        build_transaction_plan_v1, materialize_transaction_plan_v1, recheck_transaction_plan_v1,
    };

    fn id(value: u8) -> next_contracts::PersistentId {
        next_contracts::PersistentId::from_bytes([value; 16])
    }

    fn schema(value: &str) -> SchemaId {
        SchemaId::new(value).expect("test schema is valid")
    }

    fn definition(value: u8) -> DefinitionRefV1 {
        DefinitionRefV1::Exact {
            asset_id: AssetId::from_bytes([value; 16]),
            content_hash: ContentHash::from_bytes([value; 32]),
        }
    }

    fn aggregate(value: u8, payload: RpgAggregatePayloadV1) -> RpgAggregateEnvelopeV1 {
        RpgAggregateEnvelopeV1::new(
            id(value),
            1,
            0,
            definition(value),
            ProvenanceBindingV1::Exact(ContentHash::from_bytes([value; 32])),
            payload,
        )
        .expect("aggregate is valid")
    }

    fn fixture() -> RpgState {
        fixture_with_capacity(8)
    }

    fn fixture_with_capacity(capacity: u32) -> RpgState {
        let mut aggregates = vec![
            aggregate(
                1,
                RpgAggregatePayloadV1::Character(CharacterPayloadV1 {
                    inventory_id: Some(id(3)),
                    equipment_id: Some(id(4)),
                    skills: vec![],
                }),
            ),
            aggregate(
                2,
                RpgAggregatePayloadV1::Character(CharacterPayloadV1 {
                    inventory_id: None,
                    equipment_id: None,
                    skills: vec![],
                }),
            ),
            aggregate(
                5,
                RpgAggregatePayloadV1::Item(ItemPayloadV1 {
                    quantity: 1,
                    durability: 100,
                    custom_state: vec![],
                }),
            ),
            aggregate(
                3,
                RpgAggregatePayloadV1::Inventory(InventoryPayloadV1 {
                    owner_id: id(1),
                    capacity,
                    item_ids: vec![id(5)],
                    reservations: vec![],
                }),
            ),
            aggregate(
                4,
                RpgAggregatePayloadV1::Equipment(EquipmentPayloadV1 {
                    character_id: id(1),
                    slot_policy: definition(44),
                    assignments: vec![],
                }),
            ),
            aggregate(
                6,
                RpgAggregatePayloadV1::Quest(QuestPayloadV1 {
                    state_id: schema("rpg.quest.available"),
                }),
            ),
            aggregate(
                7,
                RpgAggregatePayloadV1::Dialogue(DialoguePayloadV1 {
                    speaker_id: id(1),
                    listener_id: id(2),
                    node_id: schema("rpg.dialogue.offer"),
                }),
            ),
            aggregate(
                8,
                RpgAggregatePayloadV1::Relationship(RelationshipPayloadV1 {
                    source_id: id(1),
                    target_id: id(2),
                    dimensions: vec![RelationshipDimensionV1 {
                        dimension_id: schema("rpg.relationship.trust"),
                        value: 0,
                    }],
                }),
            ),
            aggregate(
                9,
                RpgAggregatePayloadV1::Item(ItemPayloadV1 {
                    quantity: 1,
                    durability: 100,
                    custom_state: vec![],
                }),
            ),
            aggregate(
                10,
                RpgAggregatePayloadV1::InteractiveObject(InteractiveObjectPayloadV1 {
                    state_id: schema(CORE_INTERACTIVE_OBJECT_READY_STATE_ID),
                    linked_item_id: Some(id(9)),
                }),
            ),
        ];
        aggregates.sort_by_key(|aggregate| (aggregate.aggregate_kind, aggregate.persistent_id));
        RpgState::from_snapshot(RpgSnapshotV2 { aggregates }).expect("fixture is valid")
    }

    fn context<'a>(active: &'a [ContentHash]) -> RpgPlanningContextV1<'a> {
        context_with_facts(active, &[])
    }

    fn context_with_facts<'a>(
        active: &'a [ContentHash],
        physical_contact_facts: &'a [RpgPhysicalContactFactV1],
    ) -> RpgPlanningContextV1<'a> {
        RpgPlanningContextV1 {
            gameplay_tick: 0,
            causal_command_id: CommandId::from_bytes([1; 16]),
            canonical_command_body_hash: CommandBodyHash::from_bytes([2; 32]),
            project_composition_lock_hash: ContentHash::from_bytes([3; 32]),
            schema_registry_hash: ContentHash::from_bytes([4; 32]),
            budget_policy_hash: ContentHash::from_bytes([5; 32]),
            active_definition_policy_hashes: active,
            physical_contact_facts,
        }
    }

    fn target(kind: RpgAggregateKindV1, value: u8, revision: u64) -> RpgAggregateRefV1 {
        RpgAggregateRefV1 {
            aggregate_kind: kind,
            persistent_id: id(value),
            expected_revision: revision,
        }
    }

    fn dialogue_quest_command(policy: ContentHash) -> RpgCommandV1 {
        RpgCommandV1 {
            operations: vec![
                RpgOperationV1 {
                    operation_slot: 0,
                    targets: vec![target(RpgAggregateKindV1::Dialogue, 7, 0)],
                    definition_policy_hashes: vec![policy],
                    payload: RpgOperationPayloadV1::AdvanceDialogue {
                        dialogue_id: id(7),
                        expected_node_id: schema("rpg.dialogue.offer"),
                        next_node_id: schema("rpg.dialogue.accepted"),
                    },
                },
                RpgOperationV1 {
                    operation_slot: 1,
                    targets: vec![target(RpgAggregateKindV1::Quest, 6, 0)],
                    definition_policy_hashes: vec![policy],
                    payload: RpgOperationPayloadV1::TransitionQuest {
                        quest_id: id(6),
                        expected_state_id: schema("rpg.quest.available"),
                        next_state_id: schema("rpg.quest.active"),
                    },
                },
                RpgOperationV1 {
                    operation_slot: 2,
                    targets: vec![target(RpgAggregateKindV1::Relationship, 8, 0)],
                    definition_policy_hashes: vec![policy],
                    payload: RpgOperationPayloadV1::AdjustRelationship {
                        relationship_id: id(8),
                        dimension_id: schema("rpg.relationship.trust"),
                        delta: 7,
                    },
                },
            ],
        }
    }

    fn pickup_command(policy: ContentHash, expected_proxy_state: &str) -> RpgCommandV1 {
        RpgCommandV1 {
            operations: vec![
                RpgOperationV1 {
                    operation_slot: 0,
                    targets: vec![
                        target(RpgAggregateKindV1::Item, 9, 0),
                        target(RpgAggregateKindV1::Inventory, 3, 0),
                    ],
                    definition_policy_hashes: vec![policy],
                    payload: RpgOperationPayloadV1::TransferItem {
                        item_id: id(9),
                        source_inventory_id: None,
                        destination_inventory_id: Some(id(3)),
                        quantity: 1,
                    },
                },
                RpgOperationV1 {
                    operation_slot: 1,
                    targets: vec![target(RpgAggregateKindV1::InteractiveObject, 10, 0)],
                    definition_policy_hashes: vec![policy],
                    payload: RpgOperationPayloadV1::TransitionInteractiveObject {
                        object_id: id(10),
                        expected_state_id: schema(expected_proxy_state),
                        next_state_id: schema(CORE_INTERACTIVE_OBJECT_COLLECTED_STATE_ID),
                    },
                },
            ],
        }
    }

    fn pickup_fact() -> RpgPhysicalContactFactV1 {
        RpgPhysicalContactFactV1 {
            gameplay_tick: 0,
            contact_id: PhysicsContactId::from_bytes([12; 16]),
            subject_low: id(1),
            subject_high: id(10),
            physics_checkpoint_revision: 7,
            source_snapshot_hash: ContentHash::from_bytes([13; 32]),
            contact_batch_hash: ContentHash::from_bytes([14; 32]),
        }
    }

    fn assign_equipment_command(policy: ContentHash, equipment_revision: u64) -> RpgCommandV1 {
        RpgCommandV1 {
            operations: vec![RpgOperationV1 {
                operation_slot: 0,
                targets: vec![
                    target(RpgAggregateKindV1::Item, 5, 0),
                    target(RpgAggregateKindV1::Inventory, 3, 0),
                    target(RpgAggregateKindV1::Equipment, 4, equipment_revision),
                ],
                definition_policy_hashes: vec![policy],
                payload: RpgOperationPayloadV1::AssignEquipment {
                    equipment_id: id(4),
                    inventory_id: id(3),
                    item_id: id(5),
                    slot_id: schema("rpg.equipment.main-hand"),
                },
            }],
        }
    }

    #[test]
    fn build_plan_is_pure_and_repeatable() {
        let state = fixture();
        let before = state.snapshot();
        let policy = ContentHash::from_bytes([9; 32]);
        let active = [policy];
        let command = dialogue_quest_command(policy);

        let first = build_transaction_plan_v1(&state, &command, context(&active))
            .expect("first plan builds");
        let second = build_transaction_plan_v1(&state, &command, context(&active))
            .expect("second plan builds");

        assert_eq!(first, second);
        assert_eq!(state.snapshot(), before);
        assert_eq!(first.as_contract().ordered_read_set.len(), 3);
        assert_eq!(first.as_contract().ordered_write_set.len(), 3);
        assert_eq!(first.as_contract().ordered_event_drafts.len(), 3);
    }

    #[test]
    fn multi_aggregate_plan_materializes_atomically() {
        let state = fixture();
        let policy = ContentHash::from_bytes([9; 32]);
        let active = [policy];
        let plan =
            build_transaction_plan_v1(&state, &dialogue_quest_command(policy), context(&active))
                .expect("plan builds");
        let next = materialize_transaction_plan_v1(&state, &plan).expect("plan materializes");

        assert_eq!(
            next.dialogue(id(7)).expect("dialogue").node_id,
            schema("rpg.dialogue.accepted")
        );
        assert_eq!(
            next.quest(id(6)).expect("quest").state_id,
            schema("rpg.quest.active")
        );
        assert_eq!(
            next.relationship(id(8)).expect("relationship").dimensions[0].value,
            7
        );
        for kind in [
            RpgAggregateKindV1::Dialogue,
            RpgAggregateKindV1::Quest,
            RpgAggregateKindV1::Relationship,
        ] {
            let value = match kind {
                RpgAggregateKindV1::Dialogue => 7,
                RpgAggregateKindV1::Quest => 6,
                RpgAggregateKindV1::Relationship => 8,
                _ => unreachable!(),
            };
            assert_eq!(
                next.aggregate(kind, id(value)).expect("aggregate").revision,
                1
            );
        }
    }

    #[test]
    fn multiple_operations_increment_one_aggregate_once() {
        let state = fixture();
        let policy = ContentHash::from_bytes([9; 32]);
        let active = [policy];
        let command = RpgCommandV1 {
            operations: vec![
                RpgOperationV1 {
                    operation_slot: 0,
                    targets: vec![target(RpgAggregateKindV1::Relationship, 8, 0)],
                    definition_policy_hashes: vec![policy],
                    payload: RpgOperationPayloadV1::AdjustRelationship {
                        relationship_id: id(8),
                        dimension_id: schema("rpg.relationship.trust"),
                        delta: 2,
                    },
                },
                RpgOperationV1 {
                    operation_slot: 1,
                    targets: vec![target(RpgAggregateKindV1::Relationship, 8, 0)],
                    definition_policy_hashes: vec![policy],
                    payload: RpgOperationPayloadV1::AdjustRelationship {
                        relationship_id: id(8),
                        dimension_id: schema("rpg.relationship.trust"),
                        delta: 3,
                    },
                },
            ],
        };
        let plan =
            build_transaction_plan_v1(&state, &command, context(&active)).expect("plan builds");
        assert_eq!(plan.as_contract().ordered_write_set.len(), 1);
        assert_eq!(plan.as_contract().ordered_write_set[0].after_revision, 1);
        let next = materialize_transaction_plan_v1(&state, &plan).expect("plan applies");
        assert_eq!(
            next.relationship(id(8)).expect("relationship").dimensions[0].value,
            5
        );
    }

    #[test]
    fn stale_revision_and_definition_mismatch_produce_no_plan() {
        let state = fixture();
        let policy = ContentHash::from_bytes([9; 32]);
        let active = [policy];
        let mut command = dialogue_quest_command(policy);
        command.operations[0].targets[0].expected_revision = 1;
        assert!(matches!(
            build_transaction_plan_v1(&state, &command, context(&active)),
            Err(RpgPlanBuildError::RevisionStale { .. })
        ));

        let other = [ContentHash::from_bytes([10; 32])];
        assert_eq!(
            build_transaction_plan_v1(&state, &dialogue_quest_command(policy), context(&other),),
            Err(RpgPlanBuildError::DefinitionMismatch)
        );
    }

    #[test]
    fn transfer_uses_inventory_as_the_only_membership_authority() {
        let state = fixture();
        let policy = ContentHash::from_bytes([9; 32]);
        let active = [policy];
        let command = RpgCommandV1 {
            operations: vec![RpgOperationV1 {
                operation_slot: 0,
                targets: vec![
                    target(RpgAggregateKindV1::Item, 5, 0),
                    target(RpgAggregateKindV1::Inventory, 3, 0),
                ],
                definition_policy_hashes: vec![policy],
                payload: RpgOperationPayloadV1::TransferItem {
                    item_id: id(5),
                    source_inventory_id: Some(id(3)),
                    destination_inventory_id: None,
                    quantity: 1,
                },
            }],
        };
        let plan = build_transaction_plan_v1(&state, &command, context(&active))
            .expect("transfer plan builds");
        assert_eq!(plan.as_contract().ordered_write_set.len(), 1);
        assert_eq!(
            plan.as_contract().ordered_write_set[0].aggregate_kind,
            RpgAggregateKindV1::Inventory
        );
        let next = materialize_transaction_plan_v1(&state, &plan).expect("transfer applies");
        assert!(
            next.inventory(id(3))
                .expect("inventory")
                .item_ids
                .is_empty()
        );
        assert_eq!(
            next.aggregate(RpgAggregateKindV1::Item, id(5))
                .expect("item")
                .revision,
            0
        );
    }

    #[test]
    fn stale_plan_is_rejected_before_materialization() {
        let state = fixture();
        let policy = ContentHash::from_bytes([9; 32]);
        let active = [policy];
        let plan =
            build_transaction_plan_v1(&state, &dialogue_quest_command(policy), context(&active))
                .expect("plan builds");
        let next = materialize_transaction_plan_v1(&state, &plan).expect("first plan applies");
        assert_eq!(
            recheck_transaction_plan_v1(&next, &plan),
            Err(RpgPlanMaterializeError::PlanStale)
        );
    }

    #[test]
    fn failed_operation_leaves_state_byte_exact() {
        let state = fixture();
        let before = state
            .snapshot()
            .canonical_bytes()
            .expect("snapshot canonicalizes");
        let policy = ContentHash::from_bytes([9; 32]);
        let active = [policy];
        let mut command = dialogue_quest_command(policy);
        if let RpgOperationPayloadV1::TransitionQuest {
            expected_state_id, ..
        } = &mut command.operations[1].payload
        {
            *expected_state_id = schema("rpg.quest.wrong");
        }
        assert_eq!(
            build_transaction_plan_v1(&state, &command, context(&active)),
            Err(RpgPlanBuildError::TransitionInvalid)
        );
        assert_eq!(
            state
                .snapshot()
                .canonical_bytes()
                .expect("snapshot canonicalizes"),
            before
        );
    }

    #[test]
    fn pickup_requires_a_current_revision_bound_contact_fact() {
        let state = fixture();
        let policy = ContentHash::from_bytes([9; 32]);
        let active = [policy];
        let command = pickup_command(policy, CORE_INTERACTIVE_OBJECT_READY_STATE_ID);

        assert_eq!(
            build_transaction_plan_v1(&state, &command, context(&active)),
            Err(RpgPlanBuildError::PhysicalPreconditionMissing)
        );

        let fact = pickup_fact();
        let facts = [fact];
        let plan = build_transaction_plan_v1(&state, &command, context_with_facts(&active, &facts))
            .expect("contact-gated pickup plan builds");
        assert_eq!(
            plan.as_contract().validated_fact_hashes,
            vec![fact.fact_hash().expect("fact hashes")]
        );
        let next = materialize_transaction_plan_v1(&state, &plan).expect("pickup materializes");
        assert_eq!(
            next.inventory(id(3)).expect("inventory").item_ids,
            [id(5), id(9)]
        );
        assert_eq!(
            next.interactive_object(id(10)).expect("proxy").state_id,
            schema(CORE_INTERACTIVE_OBJECT_COLLECTED_STATE_ID)
        );
    }

    #[test]
    fn pickup_rejects_full_inventory_without_partial_proxy_transition() {
        let state = fixture_with_capacity(1);
        let before = state.snapshot();
        let policy = ContentHash::from_bytes([9; 32]);
        let active = [policy];
        let facts = [pickup_fact()];

        assert_eq!(
            build_transaction_plan_v1(
                &state,
                &pickup_command(policy, CORE_INTERACTIVE_OBJECT_READY_STATE_ID),
                context_with_facts(&active, &facts),
            ),
            Err(RpgPlanBuildError::OwnershipConflict)
        );
        assert_eq!(state.snapshot(), before);
    }

    #[test]
    fn pickup_fault_after_staged_transfer_is_byte_exact() {
        let state = fixture();
        let before = state
            .snapshot()
            .canonical_bytes()
            .expect("snapshot canonicalizes");
        let policy = ContentHash::from_bytes([9; 32]);
        let active = [policy];
        let facts = [pickup_fact()];

        assert_eq!(
            build_transaction_plan_v1(
                &state,
                &pickup_command(policy, "rpg.interactive.wrong"),
                context_with_facts(&active, &facts),
            ),
            Err(RpgPlanBuildError::TransitionInvalid)
        );
        assert_eq!(
            state
                .snapshot()
                .canonical_bytes()
                .expect("snapshot canonicalizes"),
            before
        );
    }

    #[test]
    fn inventory_membership_and_equipment_slot_policy_are_enforced() {
        let state = fixture();
        let policy = ContentHash::from_bytes([44; 32]);
        let active = [policy];
        let plan = build_transaction_plan_v1(
            &state,
            &assign_equipment_command(policy, 0),
            context(&active),
        )
        .expect("first assignment builds");
        let equipped =
            materialize_transaction_plan_v1(&state, &plan).expect("first assignment materializes");

        assert_eq!(
            build_transaction_plan_v1(
                &equipped,
                &assign_equipment_command(policy, 1),
                context(&active),
            ),
            Err(RpgPlanBuildError::OwnershipConflict)
        );

        let wrong_policy = ContentHash::from_bytes([45; 32]);
        assert_eq!(
            build_transaction_plan_v1(
                &state,
                &assign_equipment_command(wrong_policy, 0),
                context(&[wrong_policy]),
            ),
            Err(RpgPlanBuildError::DefinitionMismatch)
        );

        let invalid_source = RpgCommandV1 {
            operations: vec![RpgOperationV1 {
                operation_slot: 0,
                targets: vec![
                    target(RpgAggregateKindV1::Item, 9, 0),
                    target(RpgAggregateKindV1::Inventory, 3, 0),
                ],
                definition_policy_hashes: vec![],
                payload: RpgOperationPayloadV1::TransferItem {
                    item_id: id(9),
                    source_inventory_id: Some(id(3)),
                    destination_inventory_id: None,
                    quantity: 1,
                },
            }],
        };
        assert_eq!(
            build_transaction_plan_v1(&state, &invalid_source, context(&[])),
            Err(RpgPlanBuildError::OwnershipConflict)
        );
    }
}
