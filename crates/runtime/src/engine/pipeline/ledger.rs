use next_contracts::canonical::sha256;
use next_contracts::command::{CommandPhase, IssuerPrincipal, WorldCommand};
use next_contracts::ids::{CommandBodyHash, CommandId, ContentHash, content_hash_from_bytes};
use next_contracts::ledger::{
    COMMAND_RECEIPT_SCHEMA_VERSION, CommandCollisionCandidateV1, CommandCollisionIncidentV1,
    CommandFinalResultV1, CommandLedgerError, CommandReceiptSubjectV1, CommandReceiptV1,
    CommandStreamLedgerV2, CommandStreamStateV1, IdentityInsertResult,
};

use super::{PhaseContext, StagedAuthoritativeState, ValidatedCommand};
use crate::engine::error::RuntimeFatalError;
use crate::engine::result::{OrderedResult, RejectionCode};
use crate::registry::command_kind_registry_hash;

pub(super) fn handle_collision(
    context: PhaseContext<'_>,
    candidates: Vec<ValidatedCommand>,
    staged: &mut StagedAuthoritativeState,
) -> Result<Vec<OrderedResult>, RuntimeFatalError> {
    let first = candidates
        .first()
        .expect("collision group is nonempty by construction");
    if matches!(first.command.issuer, IssuerPrincipal::InternalSystem(_)) {
        return Err(RuntimeFatalError::InternalIdentityCollision);
    }
    let stream_id = first.command.stream_id;
    let sequence = first.command.sequence;
    let stream = staged
        .ledger
        .streams
        .get(&stream_id)
        .ok_or(RuntimeFatalError::LedgerCorrupt(
            CommandLedgerError::StreamKeyMismatch,
        ))?;
    if stream
        .admission_high_watermark
        .is_some_and(|high| sequence <= high)
        && !sequence_is_retained(stream, sequence)
    {
        return Ok(candidates
            .into_iter()
            .map(|candidate| {
                OrderedResult::rejected(
                    candidate.order_key,
                    candidate.command_id,
                    sequence,
                    RejectionCode::CommandSequenceFinalized,
                )
            })
            .collect());
    }
    if stream.state == CommandStreamStateV1::CollisionLocked {
        return Ok(candidates
            .into_iter()
            .map(|candidate| {
                OrderedResult::rejected(
                    candidate.order_key,
                    candidate.command_id,
                    sequence,
                    RejectionCode::CommandCollisionLocked,
                )
            })
            .collect());
    }

    let mut members = retained_collision_candidates(stream, sequence);
    for candidate in &candidates {
        insert_archive_identity(
            staged,
            &candidate.command,
            candidate.command_id,
            candidate.body_hash,
            &candidate.canonical_bytes,
        )?;
        members.push(CommandCollisionCandidateV1 {
            command_id: candidate.command_id,
            body_hash: candidate.body_hash,
            canonical_body_ref: candidate.body_hash,
        });
    }
    members.sort();
    members.dedup();
    let incident = CommandCollisionIncidentV1::new(
        stream_id,
        first.command.issuer.clone(),
        sequence,
        members,
    )?;
    let stream =
        staged
            .ledger
            .streams
            .get_mut(&stream_id)
            .ok_or(RuntimeFatalError::LedgerCorrupt(
                CommandLedgerError::StreamKeyMismatch,
            ))?;
    if sequence_is_retained(stream, sequence) {
        stream.lock_for_collision(incident)?;
    } else {
        let root =
            empty_transaction_result_root(incident.candidates_root.as_bytes(), context.phase);
        let receipt = collision_receipt(context, stream, &incident, root)?;
        stream.append_collision_receipt(incident, receipt)?;
    }
    // The complete staged ledger/archive pair is synchronized and validated
    // once at the tick commit boundary.
    Ok(candidates
        .into_iter()
        .map(|candidate| {
            OrderedResult::rejected(
                candidate.order_key,
                candidate.command_id,
                sequence,
                RejectionCode::CommandSequenceCollision,
            )
        })
        .collect())
}

pub(super) fn command_receipt(
    context: PhaseContext<'_>,
    stream: &CommandStreamLedgerV2,
    candidate: &ValidatedCommand,
    result: CommandFinalResultV1,
    event_ids: Vec<next_contracts::ids::EventId>,
    transaction_result_root: ContentHash,
) -> Result<CommandReceiptV1, RuntimeFatalError> {
    let command = &candidate.command;
    Ok(CommandReceiptV1 {
        schema_version: COMMAND_RECEIPT_SCHEMA_VERSION,
        finalization_ordinal: stream.finalized_receipt_count,
        subject: CommandReceiptSubjectV1::Command {
            stream_id: command.stream_id,
            issuer: command.issuer.clone(),
            sequence: command.sequence,
            command_id: candidate.command_id,
            body_hash: candidate.body_hash,
            canonical_body_ref: candidate.body_hash,
        },
        phase: command.phase,
        target_tick: command.target_tick,
        finalized_at_tick: context.tick,
        priority_class: candidate.order_key.priority_class,
        command_kind_registry_hash: command_kind_registry_hash(context.registry),
        result,
        diagnostic_digest: None,
        event_ids,
        transaction_result_root,
    })
}

pub(super) fn collision_receipt(
    context: PhaseContext<'_>,
    stream: &CommandStreamLedgerV2,
    incident: &CommandCollisionIncidentV1,
    transaction_result_root: ContentHash,
) -> Result<CommandReceiptV1, RuntimeFatalError> {
    Ok(CommandReceiptV1 {
        schema_version: COMMAND_RECEIPT_SCHEMA_VERSION,
        finalization_ordinal: stream.finalized_receipt_count,
        subject: CommandReceiptSubjectV1::CollisionSet {
            stream_id: incident.stream_id,
            issuer: incident.issuer.clone(),
            sequence: incident.sequence,
            candidates_root: incident.candidates_root,
            candidate_count: u32::try_from(incident.candidates.len())
                .map_err(|_| RuntimeFatalError::TraceCountExhausted)?,
            candidates: incident.candidates.clone(),
        },
        phase: context.phase,
        target_tick: context.tick,
        finalized_at_tick: context.tick,
        priority_class: u16::MAX,
        command_kind_registry_hash: command_kind_registry_hash(context.registry),
        result: CommandFinalResultV1::Collision {
            code: next_contracts::ids::SchemaId::new(
                RejectionCode::CommandSequenceCollision.as_str(),
            )
            .expect("stable collision code is valid"),
        },
        diagnostic_digest: None,
        event_ids: Vec::new(),
        transaction_result_root,
    })
}

pub(super) fn insert_archive_identity(
    staged: &mut StagedAuthoritativeState,
    command: &WorldCommand,
    command_id: CommandId,
    body_hash: CommandBodyHash,
    canonical_bytes: &[u8],
) -> Result<IdentityInsertResult, RuntimeFatalError> {
    staged.ledger_delta.stage_command_identity(
        &staged.ledger,
        &staged.archive,
        command,
        command_id,
        body_hash,
        canonical_bytes,
    )
}

pub(super) fn sequence_is_retained(stream: &CommandStreamLedgerV2, sequence: u64) -> bool {
    stream.pending.contains_key(&sequence)
        || stream
            .receipt_window
            .iter()
            .any(|receipt| receipt.subject.sequence() == sequence)
}

fn retained_collision_candidates(
    stream: &CommandStreamLedgerV2,
    sequence: u64,
) -> Vec<CommandCollisionCandidateV1> {
    if let Some(reservation) = stream.pending.get(&sequence) {
        return vec![CommandCollisionCandidateV1 {
            command_id: reservation.command_id,
            body_hash: reservation.body_hash,
            canonical_body_ref: reservation.canonical_body_ref,
        }];
    }
    stream
        .receipt_window
        .iter()
        .find(|receipt| receipt.subject.sequence() == sequence)
        .map_or_else(Vec::new, |receipt| match &receipt.subject {
            CommandReceiptSubjectV1::Command {
                command_id,
                body_hash,
                canonical_body_ref,
                ..
            } => vec![CommandCollisionCandidateV1 {
                command_id: *command_id,
                body_hash: *body_hash,
                canonical_body_ref: *canonical_body_ref,
            }],
            CommandReceiptSubjectV1::CollisionSet { candidates, .. } => candidates.clone(),
        })
}

pub(super) fn empty_transaction_result_root(identity: &[u8], phase: CommandPhase) -> ContentHash {
    transaction_result_root(identity, phase, &[], &[])
}

pub(super) fn transaction_result_root(
    identity: &[u8],
    phase: CommandPhase,
    delta_bytes: &[u8],
    event_ids: &[next_contracts::ids::EventId],
) -> ContentHash {
    let mut delta_preimage = Vec::new();
    delta_preimage.extend_from_slice(b"nextengine.transaction-deltas.v1\0");
    delta_preimage.extend_from_slice(&(delta_bytes.len() as u64).to_le_bytes());
    delta_preimage.extend_from_slice(delta_bytes);
    let delta_root = sha256(&delta_preimage);
    let mut event_preimage = Vec::new();
    event_preimage.extend_from_slice(b"nextengine.transaction-events.v1\0");
    event_preimage.extend_from_slice(&(event_ids.len() as u32).to_le_bytes());
    for event_id in event_ids {
        event_preimage.extend_from_slice(event_id.as_bytes());
    }
    let event_root = sha256(&event_preimage);
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.transaction-result.v1\0");
    preimage.extend_from_slice(identity);
    preimage.push(phase as u8);
    preimage.extend_from_slice(&delta_root);
    preimage.extend_from_slice(&event_root);
    content_hash_from_bytes(sha256(&preimage))
}
