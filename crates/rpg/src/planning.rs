use std::collections::BTreeMap;

use next_contracts::ids::{CommandBodyHash, CommandId, ContentHash, SchemaId};
use next_contracts::rpg::CORE_INTERACTIVE_OBJECT_COLLECTED_STATE_ID;
use next_contracts::rpg::{
    DefinitionRefV1, RpgAggregateEnvelopeV1, RpgAggregateKindV1, RpgAggregateRefV1, RpgCommandV1,
    RpgEventDraftV1, RpgOperationPayloadV1, RpgPhysicalContactFactV1, RpgReadSetEntryV1,
    RpgTransactionPlanV1, RpgWriteSetEntryV1,
};

use crate::operation::apply_operation;
use crate::{RpgAggregateKeyV1, RpgPlanBuildError, RpgState};

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
pub struct BuiltRpgTransactionPlanV1(pub(super) RpgTransactionPlanV1);

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
    operation: &next_contracts::rpg::RpgOperationV1,
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

fn is_subset(needles: &[ContentHash], haystack: &[ContentHash]) -> bool {
    needles
        .iter()
        .all(|needle| haystack.binary_search(needle).is_ok())
}

fn strictly_ordered_unique<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}
