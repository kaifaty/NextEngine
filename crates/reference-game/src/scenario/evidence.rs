use next_contracts::command::EventPayload;
use next_contracts::ids::{SchemaId, StateRoot};
use next_contracts::mechanics::CORE_CHARACTER_HEALTH_RESOURCE_ID;
use next_contracts::physics::ContactPhaseV1;
use next_contracts::rpg::{RpgAggregateKindV1, RpgAggregatePayloadV1, RpgPhysicalContactFactV1};
use next_runtime::RuntimeState;
use next_world::WorldRoutineOwnerV1;

use crate::ReferenceGameError;
use crate::rpg::aggregate_payload;
use crate::session::ReferenceGameSession;

use super::ReferenceStageCheckpointV2;

pub(super) fn reference_stage_checkpoint(
    runtime: &RuntimeState,
    routine: &WorldRoutineOwnerV1,
    fixture: &ReferenceGameSession,
    state_root: StateRoot,
    application_state_root: StateRoot,
    current_chunk_id: SchemaId,
) -> Result<ReferenceStageCheckpointV2, ReferenceGameError> {
    let rpg = runtime.rpg_snapshot();
    let quest_state_id = match aggregate_payload(&rpg, RpgAggregateKindV1::Quest, fixture.quest_id)
    {
        Some(RpgAggregatePayloadV1::Quest(quest)) => quest.state_id.clone(),
        _ => return Err(ReferenceGameError::RecoveryInvalid),
    };
    let dialogue_node_id =
        match aggregate_payload(&rpg, RpgAggregateKindV1::Dialogue, fixture.dialogue_id) {
            Some(RpgAggregatePayloadV1::Dialogue(dialogue)) => dialogue.node_id.clone(),
            _ => return Err(ReferenceGameError::RecoveryInvalid),
        };
    let player_inventory_contains_pickup = match aggregate_payload(
        &rpg,
        RpgAggregateKindV1::Inventory,
        fixture.player_inventory_id,
    ) {
        Some(RpgAggregatePayloadV1::Inventory(inventory)) => {
            inventory.item_ids.contains(&fixture.pickup_item_id)
        }
        _ => return Err(ReferenceGameError::RecoveryInvalid),
    };
    let player_equipment_contains_pickup = match aggregate_payload(
        &rpg,
        RpgAggregateKindV1::Equipment,
        fixture.player_equipment_id,
    ) {
        Some(RpgAggregatePayloadV1::Equipment(equipment)) => equipment
            .assignments
            .iter()
            .any(|assignment| assignment.item_id == fixture.pickup_item_id),
        _ => return Err(ReferenceGameError::RecoveryInvalid),
    };
    let health =
        |character_id| match aggregate_payload(&rpg, RpgAggregateKindV1::Character, character_id) {
            Some(RpgAggregatePayloadV1::Character(character)) => character
                .resources
                .iter()
                .find(|resource| resource.resource_id.as_str() == CORE_CHARACTER_HEALTH_RESOURCE_ID)
                .map(|resource| resource.current_value),
            _ => None,
        };
    let relay_state_id = match aggregate_payload(
        &rpg,
        RpgAggregateKindV1::InteractiveObject,
        fixture.interactive_object_id,
    ) {
        Some(RpgAggregatePayloadV1::InteractiveObject(relay)) => relay.state_id.clone(),
        _ => return Err(ReferenceGameError::RecoveryInvalid),
    };
    Ok(ReferenceStageCheckpointV2 {
        state_root,
        application_state_root,
        world_routine_activity_or_none: routine
            .snapshot_or_none()
            .map(|snapshot| snapshot.record.current_activity),
        world_routine_record_revision_or_none: routine
            .snapshot_or_none()
            .map(|snapshot| snapshot.record.record_revision),
        quest_state_id,
        dialogue_node_id,
        player_inventory_contains_pickup,
        player_equipment_contains_pickup,
        npc_health: health(fixture.npc_character_id).ok_or(ReferenceGameError::RecoveryInvalid)?,
        player_health: health(fixture.body_id).ok_or(ReferenceGameError::RecoveryInvalid)?,
        relay_state_id,
        current_chunk_id,
    })
}

#[allow(
    clippy::too_many_arguments,
    reason = "the acceptance accumulator updates the complete deterministic report"
)]
pub(super) fn accumulate_report(
    report: &next_runtime::TickReport,
    events: &mut u64,
    rpg_events: &mut u64,
    begin_contacts: &mut u64,
    persist_contacts: &mut u64,
    end_contacts: &mut u64,
    contact_preimage: &mut Vec<u8>,
) -> Result<(), ReferenceGameError> {
    *events = events
        .checked_add(
            u64::try_from(report.events.len()).map_err(|_| ReferenceGameError::CountOverflow)?,
        )
        .ok_or(ReferenceGameError::CountOverflow)?;
    *rpg_events = rpg_events
        .checked_add(
            u64::try_from(
                report
                    .events
                    .iter()
                    .filter(|event| matches!(&event.payload, EventPayload::Rpg(_)))
                    .count(),
            )
            .map_err(|_| ReferenceGameError::CountOverflow)?,
        )
        .ok_or(ReferenceGameError::CountOverflow)?;
    for contact in &report.contact_batch.events {
        let count = match contact.phase {
            ContactPhaseV1::Begin => &mut *begin_contacts,
            ContactPhaseV1::Persist => &mut *persist_contacts,
            ContactPhaseV1::End => &mut *end_contacts,
        };
        *count = count
            .checked_add(1)
            .ok_or(ReferenceGameError::CountOverflow)?;
    }
    contact_preimage.extend_from_slice(report.contact_batch.batch_hash.as_bytes());
    Ok(())
}

pub(super) fn rpg_contact_facts_from_report(
    batch: &next_contracts::physics::ClosedPhysicsContactBatchV1,
    physics_checkpoint_revision: u64,
) -> Vec<RpgPhysicalContactFactV1> {
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
    facts
}
