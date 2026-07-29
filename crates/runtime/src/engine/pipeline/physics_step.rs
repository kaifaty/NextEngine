use std::collections::BTreeMap;

use next_contracts::command::DomainEvent;
use next_contracts::ledger::{
    CausalIdentityKind, CommandFinalResultV1, CommandLedgerError, IdentityInsertResult,
};
use next_contracts::physics::{
    PHYSICS_STEP_INPUT_SCHEMA_VERSION, PhysicalEventV1, PhysicsStepInputV2,
};

use super::ledger::{command_receipt, transaction_result_root};
use super::{PhaseContext, PhysicalPending, PhysicalStepExecution, StagedAuthoritativeState};
use crate::engine::error::RuntimeFatalError;
use crate::engine::result::OrderedResult;

pub(super) fn finish_physical_step(
    context: PhaseContext<'_>,
    pending: Vec<PhysicalPending>,
    staged: &mut StagedAuthoritativeState,
) -> Result<PhysicalStepExecution, RuntimeFatalError> {
    let before_snapshot = staged.physics.snapshot().clone();
    let mut intents = pending
        .iter()
        .map(|pending| pending.intent.clone())
        .collect::<Vec<_>>();
    intents.sort_by_key(|intent| (intent.body_id, intent.causal_command_id));
    let step_input = PhysicsStepInputV2 {
        schema_version: PHYSICS_STEP_INPUT_SCHEMA_VERSION,
        world_id: before_snapshot.world_id,
        expected_world_revision: before_snapshot.world_revision,
        expected_snapshot_hash: before_snapshot.snapshot_hash()?,
        expected_catalog_hash: staged.physics.checkpoint().catalog.catalog_hash()?,
        gameplay_tick: context.tick,
        first_physics_tick: before_snapshot
            .physics_tick
            .checked_add(1)
            .ok_or(RuntimeFatalError::PhysicalOutcomeInvariant)?,
        physics_substeps: staged
            .physics
            .tick_rate_profile()
            .physics_substeps_per_gameplay_tick,
        accepted_intents: intents,
    };

    let step_result = staged.physics.step(&step_input)?;
    if step_result.applied_locomotion.len() != pending.len() {
        return Err(RuntimeFatalError::PhysicalOutcomeInvariant);
    }
    if step_result.before_snapshot_hash != step_result.after_snapshot_hash {
        staged.revision = staged
            .revision
            .checked_add(1)
            .ok_or(RuntimeFatalError::RevisionExhausted)?;
    }
    let applied_by_command = step_result
        .applied_locomotion
        .into_iter()
        .map(|applied| (applied.causal_command_id, applied))
        .collect::<BTreeMap<_, _>>();

    let mut results = Vec::with_capacity(pending.len());
    for pending in pending {
        let applied = applied_by_command
            .get(&pending.candidate.command_id)
            .ok_or(RuntimeFatalError::PhysicalOutcomeInvariant)?;
        if pending.intent.body_id != applied.body_id {
            return Err(RuntimeFatalError::PhysicalOutcomeInvariant);
        }
        let before = before_snapshot
            .sorted_body_states
            .get(&applied.body_id)
            .ok_or(RuntimeFatalError::PhysicalOutcomeInvariant)?;
        let after = staged
            .physics
            .snapshot()
            .sorted_body_states
            .get(&applied.body_id)
            .ok_or(RuntimeFatalError::PhysicalOutcomeInvariant)?;
        let command = &pending.candidate.command;
        let event = (applied.applied_displacement_micrometres[0] != 0
            || applied.applied_displacement_micrometres[2] != 0)
            .then(|| {
                DomainEvent::physical(
                    context.tick,
                    context.phase,
                    pending.candidate.command_id,
                    0,
                    PhysicalEventV1::CapsuleStepApplied {
                        body_id: pending.intent.controlled_target_id,
                        physics_tick: staged.physics.snapshot().physics_tick,
                        before: before.pose,
                        after: after.pose,
                    },
                )
            })
            .transpose()?;
        let event_ids = event
            .as_ref()
            .map_or_else(Vec::new, |event| vec![event.event_id]);
        if let Some(event) = &event {
            let provenance = event.canonical_bytes()?;
            let insert = staged
                .ledger
                .causal_identity_registry
                .compare_or_insert_provenance(
                    CausalIdentityKind::DomainEvent,
                    *event.event_id.as_bytes(),
                    &provenance,
                )?;
            if insert == IdentityInsertResult::Collision {
                return Err(RuntimeFatalError::InternalIdentityCollision);
            }
        }
        let delta = if pending.intent.direction_q15 == [0, 0] {
            Vec::new()
        } else {
            applied.canonical_bytes()?
        };
        let transaction_root = transaction_result_root(
            pending.candidate.command_id.as_bytes(),
            context.phase,
            &delta,
            &event_ids,
        );
        let receipt = command_receipt(
            context,
            staged.ledger.streams.get(&command.stream_id).ok_or(
                RuntimeFatalError::LedgerCorrupt(CommandLedgerError::StreamKeyMismatch),
            )?,
            command,
            pending.candidate.command_id,
            CommandFinalResultV1::Committed,
            event_ids,
            transaction_root,
        )?;
        staged
            .ledger
            .streams
            .get_mut(&command.stream_id)
            .ok_or(RuntimeFatalError::LedgerCorrupt(
                CommandLedgerError::StreamKeyMismatch,
            ))?
            .append_receipt(receipt)?;
        if event.is_some() {
            staged.event_count = staged
                .event_count
                .checked_add(1)
                .ok_or(RuntimeFatalError::EventCountExhausted)?;
        }
        results.push((
            OrderedResult::committed(
                pending.candidate.order_key,
                pending.candidate.command_id,
                command.sequence,
            ),
            event,
        ));
    }
    Ok(PhysicalStepExecution {
        command_results: results,
        step_input,
        contact_batch: step_result.contact_batch,
    })
}
