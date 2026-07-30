use std::collections::BTreeSet;

use next_contracts::canonical::CanonicalError;
use next_contracts::command::{CommandPayload, DomainEvent, IssuerPrincipal};
use next_contracts::ledger::{
    CausalIdentityKind, CommandCollisionCandidateV1, CommandCollisionIncidentV1,
    CommandFinalResultV1, CommandLedgerError, CommandReceiptSubjectV1, CommandReservationV1,
    CommandStreamLedgerV2, CommandStreamStateV1, IdentityInsertResult,
};
use next_contracts::physics::{AcceptedLocomotionIntentV2, PhysicalCommandV1};
use next_contracts::rpg::RpgTransactionPlanV1;
use next_rpg::{
    RpgPlanBuildError, RpgPlanMaterializeError, RpgPlanningContextV1, build_transaction_plan_v1,
    materialize_transaction_plan_v1,
};

use super::ledger::{
    collision_receipt, command_receipt, empty_transaction_result_root, handle_collision,
    insert_archive_identity, sequence_is_retained, transaction_result_root,
};
use super::{
    CandidateExecution, ExecutionTrace, PhaseContext, PhysicalPending, StagedAuthoritativeState,
    ValidatedCommand,
};
use crate::engine::error::RuntimeFatalError;
use crate::engine::result::{CommittedRpgPlanTraceV1, OrderedResult, RejectionCode};

pub(super) fn execute_candidate(
    context: PhaseContext<'_>,
    candidate: ValidatedCommand,
    staged: &mut StagedAuthoritativeState,
    physical_bodies: &mut BTreeSet<next_contracts::ids::PersistentId>,
) -> Result<CandidateExecution, RuntimeFatalError> {
    let command = &candidate.command;
    let stream =
        staged
            .ledger
            .streams
            .get(&command.stream_id)
            .ok_or(RuntimeFatalError::LedgerCorrupt(
                CommandLedgerError::StreamKeyMismatch,
            ))?;
    if let Some(result) = retained_result(stream, &candidate)? {
        return Ok(CandidateExecution::Result(
            result,
            ExecutionTrace::Deduplicated,
        ));
    }
    if stream
        .receipt_window
        .iter()
        .any(|receipt| receipt.subject.sequence() == command.sequence)
    {
        return collision_with_retained(context, candidate, staged);
    }
    if let Some(reservation) = stream.pending.get(&command.sequence) {
        if reservation.command_id == candidate.command_id
            && reservation.body_hash == command.body_hash()?
        {
            if !candidate.due {
                return Ok(CandidateExecution::Result(
                    OrderedResult::reserved(
                        candidate.order_key,
                        candidate.command_id,
                        command.sequence,
                    ),
                    ExecutionTrace::Deduplicated,
                ));
            }
        } else {
            return collision_with_retained(context, candidate, staged);
        }
    } else if stream
        .admission_high_watermark
        .is_some_and(|high| command.sequence <= high)
    {
        return Ok(CandidateExecution::Result(
            OrderedResult::rejected(
                candidate.order_key,
                candidate.command_id,
                command.sequence,
                RejectionCode::CommandSequenceFinalized,
            ),
            ExecutionTrace::Rejected,
        ));
    }
    match stream.state {
        CommandStreamStateV1::CollisionLocked => {
            return Ok(CandidateExecution::Result(
                OrderedResult::rejected(
                    candidate.order_key,
                    candidate.command_id,
                    command.sequence,
                    RejectionCode::CommandCollisionLocked,
                ),
                ExecutionTrace::Rejected,
            ));
        }
        CommandStreamStateV1::Closed => {
            return Ok(CandidateExecution::Result(
                OrderedResult::rejected(
                    candidate.order_key,
                    candidate.command_id,
                    command.sequence,
                    RejectionCode::CommandStreamClosed,
                ),
                ExecutionTrace::Rejected,
            ));
        }
        CommandStreamStateV1::Exhausted => {
            return Ok(CandidateExecution::Result(
                OrderedResult::rejected(
                    candidate.order_key,
                    candidate.command_id,
                    command.sequence,
                    RejectionCode::CommandSequenceExhausted,
                ),
                ExecutionTrace::Rejected,
            ));
        }
        CommandStreamStateV1::Open => {}
    }

    if !candidate.due
        && staged
            .ledger
            .streams
            .get(&command.stream_id)
            .is_some_and(|stream| {
                stream.pending.len() >= next_contracts::ledger::COMMAND_PENDING_CAPACITY
            })
    {
        return Ok(CandidateExecution::Result(
            OrderedResult::rejected(
                candidate.order_key,
                candidate.command_id,
                command.sequence,
                RejectionCode::CommandPendingLimit,
            ),
            ExecutionTrace::Rejected,
        ));
    }

    if !candidate.due {
        let identity_result = insert_archive_identity(
            &mut staged.ledger,
            &mut staged.archive,
            command,
            candidate.command_id,
        )?;
        if identity_result == IdentityInsertResult::Collision {
            return collision_from_identity_index(context, candidate, staged);
        }
        // The staged archive manifest is synchronized once after both command
        // phases. Per-candidate synchronization rescans the complete retained
        // ledger and turns a fixed-rate live tick into work proportional to
        // the entire session history.
        let descriptor = context
            .registry
            .descriptor(&command.payload_schema_id, command.payload_schema_version)
            .expect("pre-ledger schema validation resolved the descriptor");
        if command.phase != context.phase || !descriptor.allows_phase(command.phase) {
            return finalize_rejection(
                context,
                candidate,
                staged,
                RejectionCode::CommandPhaseForbidden,
            );
        }
        match &command.payload {
            CommandPayload::Physical(_) if command.target.is_none() => {
                return finalize_rejection(
                    context,
                    candidate,
                    staged,
                    RejectionCode::PhysicalTargetUnbound,
                );
            }
            CommandPayload::Physical(_) => {}
            _ if command.target.is_some() => {
                return finalize_rejection(
                    context,
                    candidate,
                    staged,
                    RejectionCode::TargetNotAllowed,
                );
            }
            _ => {}
        }
        if command.target_tick < context.tick {
            return finalize_rejection(context, candidate, staged, RejectionCode::CommandExpired);
        }
        let horizon = context
            .tick
            .checked_add(u64::from(context.profile.maximum_future_command_ticks))
            .ok_or(RuntimeFatalError::TickExhausted)?;
        if command.target_tick > horizon {
            return finalize_rejection(
                context,
                candidate,
                staged,
                RejectionCode::CommandFutureLimit,
            );
        }
        let reservation = CommandReservationV1::from_command(
            command,
            context.tick,
            candidate.order_key.priority_class,
            context.registry.canonical_hash(),
        )?;
        let reserve = staged
            .ledger
            .streams
            .get_mut(&command.stream_id)
            .ok_or(RuntimeFatalError::LedgerCorrupt(
                CommandLedgerError::StreamKeyMismatch,
            ))?
            .reserve(reservation);
        if let Err(CommandLedgerError::TargetTickRegression) = reserve {
            return finalize_rejection(
                context,
                candidate,
                staged,
                RejectionCode::CommandStreamTimeRegression,
            );
        }
        reserve?;
        if command.target_tick > context.tick {
            return Ok(CandidateExecution::Result(
                OrderedResult::reserved(
                    candidate.order_key,
                    candidate.command_id,
                    command.sequence,
                ),
                ExecutionTrace::Reserved,
            ));
        }
    }

    if command
        .precondition_revision()
        .is_some_and(|revision| revision != context.phase_revision)
    {
        return finalize_rejection(
            context,
            candidate,
            staged,
            RejectionCode::PreconditionFailed,
        );
    }

    if let CommandPayload::Physical(PhysicalCommandV1::SetCapsuleLocomotionIntent {
        direction_q15,
    }) = &command.payload
    {
        let body_id = command
            .target
            .expect("physical target presence was validated after admission");
        let binding_matches = context.controllers.bindings.values().any(|binding| {
            binding.principal == command.issuer
                && binding.command_stream_id == command.stream_id
                && binding.controlled_body_id == body_id
        });
        if !binding_matches {
            return finalize_rejection(
                context,
                candidate,
                staged,
                RejectionCode::PhysicalTargetUnbound,
            );
        }
        let Some(physics_body_id) = staged
            .physics
            .checkpoint()
            .catalog
            .avatar_bindings
            .get(&body_id)
            .copied()
        else {
            return finalize_rejection(
                context,
                candidate,
                staged,
                RejectionCode::PhysicalTargetUnbound,
            );
        };
        let Some(body) = staged
            .physics
            .snapshot()
            .sorted_body_states
            .get(&physics_body_id)
        else {
            return finalize_rejection(
                context,
                candidate,
                staged,
                RejectionCode::PhysicalTargetUnbound,
            );
        };
        if !body.active {
            return finalize_rejection(
                context,
                candidate,
                staged,
                RejectionCode::PhysicalBodyInactive,
            );
        }
        if !physical_bodies.insert(body_id) {
            return finalize_rejection(
                context,
                candidate,
                staged,
                RejectionCode::PhysicalIntentAlreadyAssigned,
            );
        }
        let intent = AcceptedLocomotionIntentV2 {
            causal_command_id: candidate.command_id,
            controlled_target_id: body_id,
            body_id: physics_body_id,
            target_gameplay_tick: context.tick,
            direction_q15: *direction_q15,
        };
        return Ok(CandidateExecution::PhysicalPending(Box::new(
            PhysicalPending { candidate, intent },
        )));
    }

    let (after_rpg, events, delta, plan_trace) = match &command.payload {
        CommandPayload::Noop => (
            staged.rpg.clone(),
            vec![DomainEvent::command_committed(
                context.tick,
                context.phase,
                candidate.command_id,
                command.sequence,
            )?],
            Vec::new(),
            None,
        ),
        CommandPayload::Rpg(rpg_command) => {
            let planning_context = RpgPlanningContextV1 {
                gameplay_tick: context.tick,
                causal_command_id: candidate.command_id,
                canonical_command_body_hash: command.body_hash()?,
                project_composition_lock_hash: context.rpg_bindings.project_composition_lock_hash,
                schema_registry_hash: context.rpg_bindings.schema_registry_hash,
                budget_policy_hash: context.rpg_bindings.budget_policy_hash,
                active_definition_policy_hashes: &context
                    .rpg_bindings
                    .active_definition_policy_hashes,
                physical_contact_facts: context.physical_contact_facts,
            };
            let plan = match build_transaction_plan_v1(&staged.rpg, rpg_command, planning_context) {
                Ok(plan) => plan,
                Err(error) => {
                    return finalize_rejection(
                        context,
                        candidate,
                        staged,
                        rpg_rejection_code(&error),
                    );
                }
            };
            let next_rpg = match materialize_transaction_plan_v1(&staged.rpg, &plan) {
                Ok(state) => state,
                Err(RpgPlanMaterializeError::PlanStale) => {
                    return finalize_rejection(
                        context,
                        candidate,
                        staged,
                        RejectionCode::RpgPlanStale,
                    );
                }
                Err(_) => {
                    return finalize_rejection(
                        context,
                        candidate,
                        staged,
                        RejectionCode::RpgTransactionAborted,
                    );
                }
            };
            let mut events = Vec::with_capacity(plan.as_contract().ordered_event_drafts.len());
            for (event_slot, draft) in plan.as_contract().ordered_event_drafts.iter().enumerate() {
                events.push(DomainEvent::rpg(
                    context.tick,
                    context.phase,
                    candidate.command_id,
                    u32::try_from(event_slot)
                        .map_err(|_| RuntimeFatalError::EventCountExhausted)?,
                    draft.event.clone(),
                )?);
            }
            let delta = canonical_rpg_plan_delta(plan.as_contract())?;
            let trace = CommittedRpgPlanTraceV1 {
                command_id: candidate.command_id,
                plan_hash: plan.as_contract().plan_hash,
            };
            (next_rpg, events, delta, Some(trace))
        }
        CommandPayload::Physical(_) => {
            unreachable!("physical commands return a pending step before domain execution")
        }
    };
    let mut next_ledger = staged.ledger.clone();
    for event in &events {
        let provenance = event.canonical_bytes()?;
        let insert = next_ledger
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
    let event_ids = events
        .iter()
        .map(|event| event.event_id)
        .collect::<Vec<_>>();
    let transaction_root = transaction_result_root(
        candidate.command_id.as_bytes(),
        context.phase,
        &delta,
        &event_ids,
    );
    let receipt = command_receipt(
        context,
        next_ledger
            .streams
            .get(&command.stream_id)
            .ok_or(RuntimeFatalError::LedgerCorrupt(
                CommandLedgerError::StreamKeyMismatch,
            ))?,
        command,
        candidate.command_id,
        CommandFinalResultV1::Committed,
        event_ids,
        transaction_root,
    )?;
    next_ledger
        .streams
        .get_mut(&command.stream_id)
        .ok_or(RuntimeFatalError::LedgerCorrupt(
            CommandLedgerError::StreamKeyMismatch,
        ))?
        .append_receipt(receipt)?;
    let event_increment =
        u64::try_from(events.len()).map_err(|_| RuntimeFatalError::EventCountExhausted)?;
    let next_event_count = staged
        .event_count
        .checked_add(event_increment)
        .ok_or(RuntimeFatalError::EventCountExhausted)?;
    let next_revision = staged
        .revision
        .checked_add(1)
        .ok_or(RuntimeFatalError::RevisionExhausted)?;
    staged.ledger = next_ledger;
    staged.event_count = next_event_count;
    staged.revision = next_revision;
    staged.rpg = after_rpg;
    Ok(CandidateExecution::Committed(
        OrderedResult::committed(candidate.order_key, candidate.command_id, command.sequence),
        events,
        plan_trace,
    ))
}

fn retained_result(
    stream: &CommandStreamLedgerV2,
    candidate: &ValidatedCommand,
) -> Result<Option<OrderedResult>, RuntimeFatalError> {
    let command = &candidate.command;
    let body_hash = command.body_hash()?;
    let Some(receipt) = stream
        .receipt_window
        .iter()
        .find(|receipt| receipt.subject.sequence() == command.sequence)
    else {
        return Ok(None);
    };
    let exact = match &receipt.subject {
        CommandReceiptSubjectV1::Command {
            command_id,
            body_hash: retained_hash,
            ..
        } => *command_id == candidate.command_id && *retained_hash == body_hash,
        CommandReceiptSubjectV1::CollisionSet { candidates, .. } => {
            candidates.iter().any(|member| {
                member.command_id == candidate.command_id && member.body_hash == body_hash
            })
        }
    };
    if !exact {
        return Ok(None);
    }
    let result = match &receipt.result {
        CommandFinalResultV1::Committed => OrderedResult::deduplicated(
            candidate.order_key.clone(),
            candidate.command_id,
            command.sequence,
        ),
        CommandFinalResultV1::Rejected { code } => OrderedResult::rejected(
            candidate.order_key.clone(),
            candidate.command_id,
            command.sequence,
            RejectionCode::from_stable_code(code.as_str()),
        ),
        CommandFinalResultV1::Collision { .. } => OrderedResult::rejected(
            candidate.order_key.clone(),
            candidate.command_id,
            command.sequence,
            RejectionCode::CommandSequenceCollision,
        ),
    };
    Ok(Some(result))
}

fn collision_with_retained(
    context: PhaseContext<'_>,
    candidate: ValidatedCommand,
    staged: &mut StagedAuthoritativeState,
) -> Result<CandidateExecution, RuntimeFatalError> {
    let results = handle_collision(context, vec![candidate], staged)?;
    Ok(CandidateExecution::Result(
        results
            .into_iter()
            .next()
            .expect("one collision candidate returns one result"),
        ExecutionTrace::Rejected,
    ))
}

fn collision_from_identity_index(
    context: PhaseContext<'_>,
    candidate: ValidatedCommand,
    staged: &mut StagedAuthoritativeState,
) -> Result<CandidateExecution, RuntimeFatalError> {
    if matches!(candidate.command.issuer, IssuerPrincipal::InternalSystem(_)) {
        return Err(RuntimeFatalError::InternalIdentityCollision);
    }
    let binding = staged
        .ledger
        .identity_index
        .body
        .bindings
        .get(&candidate.command_id)
        .ok_or(RuntimeFatalError::LedgerCorrupt(
            CommandLedgerError::IdentityReferenceMissing,
        ))?;
    let members = binding
        .occurrences
        .iter()
        .map(|occurrence| CommandCollisionCandidateV1 {
            command_id: candidate.command_id,
            body_hash: occurrence.body_hash,
            canonical_body_ref: occurrence.body_hash,
        })
        .collect();
    let incident = CommandCollisionIncidentV1::new(
        candidate.command.stream_id,
        candidate.command.issuer.clone(),
        candidate.command.sequence,
        members,
    )?;
    let stream = staged
        .ledger
        .streams
        .get_mut(&candidate.command.stream_id)
        .ok_or(RuntimeFatalError::LedgerCorrupt(
            CommandLedgerError::StreamKeyMismatch,
        ))?;
    if sequence_is_retained(stream, candidate.command.sequence) {
        stream.lock_for_collision(incident)?;
    } else {
        let root =
            empty_transaction_result_root(incident.candidates_root.as_bytes(), context.phase);
        let receipt = collision_receipt(context, stream, &incident, root)?;
        stream.append_collision_receipt(incident, receipt)?;
    }
    Ok(CandidateExecution::Result(
        OrderedResult::rejected(
            candidate.order_key,
            candidate.command_id,
            candidate.command.sequence,
            RejectionCode::CommandIdCollision,
        ),
        ExecutionTrace::Rejected,
    ))
}

fn finalize_rejection(
    context: PhaseContext<'_>,
    candidate: ValidatedCommand,
    staged: &mut StagedAuthoritativeState,
    code: RejectionCode,
) -> Result<CandidateExecution, RuntimeFatalError> {
    let command = &candidate.command;
    let root = empty_transaction_result_root(candidate.command_id.as_bytes(), context.phase);
    let receipt =
        command_receipt(
            context,
            staged.ledger.streams.get(&command.stream_id).ok_or(
                RuntimeFatalError::LedgerCorrupt(CommandLedgerError::StreamKeyMismatch),
            )?,
            command,
            candidate.command_id,
            CommandFinalResultV1::Rejected {
                code: next_contracts::ids::SchemaId::new(code.as_str())
                    .expect("stable rejection code is a valid identifier"),
            },
            Vec::new(),
            root,
        )?;
    staged
        .ledger
        .streams
        .get_mut(&command.stream_id)
        .ok_or(RuntimeFatalError::LedgerCorrupt(
            CommandLedgerError::StreamKeyMismatch,
        ))?
        .append_receipt(receipt)?;
    Ok(CandidateExecution::Result(
        OrderedResult::rejected(
            candidate.order_key,
            candidate.command_id,
            command.sequence,
            code,
        ),
        ExecutionTrace::Rejected,
    ))
}

fn canonical_rpg_plan_delta(plan: &RpgTransactionPlanV1) -> Result<Vec<u8>, CanonicalError> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"nextengine.rpg-owner-write-set.v1\0");
    bytes.extend_from_slice(
        &u32::try_from(plan.ordered_write_set.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for write in &plan.ordered_write_set {
        bytes.push(write.aggregate_kind as u8);
        bytes.extend_from_slice(write.persistent_id.as_bytes());
        bytes.extend_from_slice(&write.before_revision.to_le_bytes());
        bytes.extend_from_slice(&write.after_revision.to_le_bytes());
        bytes.extend_from_slice(write.before_state_hash.as_bytes());
        bytes.extend_from_slice(write.after_state_hash.as_bytes());
        let after = write.after.canonical_bytes()?;
        bytes.extend_from_slice(
            &u32::try_from(after.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        bytes.extend_from_slice(&after);
    }
    Ok(bytes)
}

fn rpg_rejection_code(error: &RpgPlanBuildError) -> RejectionCode {
    match error {
        RpgPlanBuildError::Contract(error) => RejectionCode::from_stable_code(error.stable_code()),
        RpgPlanBuildError::AggregateNotFound(_) => RejectionCode::RpgAggregateNotFound,
        RpgPlanBuildError::RevisionStale { .. } => RejectionCode::RpgRevisionStale,
        RpgPlanBuildError::RevisionExhausted => RejectionCode::RpgRevisionExhausted,
        RpgPlanBuildError::TransitionInvalid => RejectionCode::RpgTransitionInvalid,
        RpgPlanBuildError::OwnershipConflict => RejectionCode::RpgOwnershipConflict,
        RpgPlanBuildError::ReservationInvalid => RejectionCode::RpgReservationInvalid,
        RpgPlanBuildError::PhysicalPreconditionMissing => {
            RejectionCode::RpgPhysicalPreconditionMissing
        }
        RpgPlanBuildError::DefinitionMismatch => RejectionCode::RpgDefinitionMismatch,
        RpgPlanBuildError::CommitmentRejected => RejectionCode::RpgCommitmentRejected,
        RpgPlanBuildError::TransactionAborted => RejectionCode::RpgTransactionAborted,
        _ => RejectionCode::RpgTransactionAborted,
    }
}
