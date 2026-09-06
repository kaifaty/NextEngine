use std::collections::BTreeSet;

use next_contracts::canonical::CanonicalError;
use next_contracts::command::{CommandPayload, DomainEvent, IssuerPrincipal};
use next_contracts::ledger::{
    COMMAND_RESERVATION_SCHEMA_VERSION, CommandCollisionCandidateV1, CommandCollisionIncidentV1,
    CommandFinalResultV1, CommandLedgerError, CommandReceiptSubjectV1, CommandReservationV1,
    CommandStreamLedgerV2, CommandStreamStateV1, IdentityInsertResult,
};
use next_contracts::physical_animation::{
    RootMotionIntentV1, capsule_root_motion_profile_hash_v1,
    capsule_root_motion_step_micrometres_v1,
};
use next_contracts::physics::{
    AcceptedLocomotionIntentV2, PhysicalCommandV1, WaterFlowRejectionV1, WaterVolumeRejectionV1,
};
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
use crate::engine::agent_cognition::AgentCognitionStageContextV1;
use crate::engine::error::RuntimeFatalError;
use crate::engine::result::{CommittedRpgPlanTraceV1, OrderedResult, RejectionCode};
use crate::engine::world_activity::WorldActivityStageContextV1;
use crate::engine::world_population::WorldPopulationStageContextV1;
use crate::engine::world_routine::WorldRoutineStageContextV1;

fn validate_root_motion_admission(
    intent: &RootMotionIntentV1,
    command: &next_contracts::command::WorldCommand,
    body_revision: u64,
    context: PhaseContext<'_>,
) -> Result<[i16; 2], RejectionCode> {
    // The saved locomotion graph phase can start between world-tick cadence
    // residues. V1 records the sampled interval but not that phase cursor, so
    // Runtime admits exactly the two legal fixed-rate quanta rather than
    // pretending it can derive clip-phase parity from the world tick.
    let expected_interval_floor = 1_000_000_u64 / u64::from(context.gameplay_hz);
    let expected_interval_ceil = 1_000_000_u64.div_ceil(u64::from(context.gameplay_hz));
    let expected_step = capsule_root_motion_step_micrometres_v1(context.gameplay_hz)
        .map_err(|_| RejectionCode::RootMotionIntentRejected)?;
    let expected_profile = capsule_root_motion_profile_hash_v1(context.gameplay_hz)
        .map_err(|_| RejectionCode::RootMotionIntentRejected)?;
    if intent.validate().is_err()
        || command.target != Some(intent.subject_id)
        || intent.intent_sequence != command.sequence
        || context
            .authority
            .root_motion_source(&command.issuer, intent.subject_id)
            != Some((intent.source_graph_hash, intent.source_clip_hash))
        || intent.source_animation_tick != context.tick
        || intent.expected_intent_state_revision != context.tick
        || intent.expected_body_revision != body_revision
        || intent.locomotion_profile_hash != expected_profile
        || intent.quantized_local_translation != [0, 0, expected_step]
        || !matches!(
            intent.interval_us,
            value if value == expected_interval_floor || value == expected_interval_ceil
        )
    {
        return Err(RejectionCode::RootMotionIntentRejected);
    }
    Ok([0, 32_767])
}
use crate::registry::command_kind_registry_hash;

#[allow(
    clippy::too_many_arguments,
    reason = "domain execution receives each optional staged authoritative owner explicitly"
)]
pub(super) fn execute_candidate(
    context: PhaseContext<'_>,
    candidate: ValidatedCommand,
    staged: &mut StagedAuthoritativeState,
    physical_bodies: &mut BTreeSet<next_contracts::ids::PersistentId>,
    world_routine: Option<&mut WorldRoutineStageContextV1>,
    world_population: Option<&mut WorldPopulationStageContextV1>,
    world_activity: Option<&mut WorldActivityStageContextV1>,
    agent_cognition: Option<&mut AgentCognitionStageContextV1>,
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
    // A sequence above the persisted high-watermark cannot already exist in
    // either the retained receipt suffix or the pending reservation map. The
    // ordinary monotonic stream path therefore avoids two linear history
    // scans; retry, collision and finalized-gap semantics keep the exact
    // retained-history checks below.
    if stream
        .admission_high_watermark
        .is_some_and(|high| command.sequence <= high)
    {
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
                && reservation.body_hash == candidate.body_hash
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
        } else {
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
            staged,
            command,
            candidate.command_id,
            candidate.body_hash,
            &candidate.canonical_bytes,
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
            CommandPayload::RootMotion(_) if command.target.is_none() => {
                return finalize_rejection(
                    context,
                    candidate,
                    staged,
                    RejectionCode::PhysicalTargetUnbound,
                );
            }
            CommandPayload::RootMotion(_) => {}
            CommandPayload::WorldRoutine(_) if command.target.is_none() => {
                return finalize_rejection(
                    context,
                    candidate,
                    staged,
                    RejectionCode::TargetNotAllowed,
                );
            }
            CommandPayload::WorldRoutine(_) => {}
            CommandPayload::WorldPopulation(_) if command.target.is_none() => {
                return finalize_rejection(
                    context,
                    candidate,
                    staged,
                    RejectionCode::TargetNotAllowed,
                );
            }
            CommandPayload::WorldPopulation(_) => {}
            CommandPayload::WorldActivity(_) if command.target.is_none() => {
                return finalize_rejection(
                    context,
                    candidate,
                    staged,
                    RejectionCode::TargetNotAllowed,
                );
            }
            CommandPayload::WorldActivity(_) => {}
            CommandPayload::AgentCognition(_) if command.target.is_none() => {
                return finalize_rejection(
                    context,
                    candidate,
                    staged,
                    RejectionCode::TargetNotAllowed,
                );
            }
            CommandPayload::AgentCognition(_) => {}
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
        let reservation = CommandReservationV1 {
            schema_version: COMMAND_RESERVATION_SCHEMA_VERSION,
            stream_id: command.stream_id,
            issuer: command.issuer.clone(),
            sequence: command.sequence,
            command_id: candidate.command_id,
            body_hash: candidate.body_hash,
            canonical_body_ref: candidate.body_hash,
            reserved_at_tick: context.tick,
            target_tick: command.target_tick,
            phase: command.phase,
            priority_class: candidate.order_key.priority_class,
            command_kind_registry_hash: command_kind_registry_hash(context.registry),
        };
        let reserve = staged
            .ledger
            .streams
            .get_mut(&command.stream_id)
            .ok_or(RuntimeFatalError::LedgerCorrupt(
                CommandLedgerError::StreamKeyMismatch,
            ))?
            .reserve_incremental(reservation);
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

    let physical_direction = match &command.payload {
        CommandPayload::Physical(PhysicalCommandV1::SetCapsuleLocomotionIntent {
            direction_q15,
        }) => Some(*direction_q15),
        CommandPayload::RootMotion(_) => Some([0, 32_767]),
        _ => None,
    };
    if let Some(mut direction_q15) = physical_direction {
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
        if let CommandPayload::RootMotion(intent) = &command.payload {
            match validate_root_motion_admission(intent, command, body.body_revision, context) {
                Ok(direction) => direction_q15 = direction,
                Err(code) => return finalize_rejection(context, candidate, staged, code),
            }
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
            direction_q15,
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
            if let Some(routine) = world_routine {
                routine.validate_interaction_command(candidate.command_id)?;
            }
            let planning_context = RpgPlanningContextV1 {
                gameplay_tick: context.tick,
                causal_command_id: candidate.command_id,
                canonical_command_body_hash: candidate.body_hash,
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
        CommandPayload::RootMotion(_) => {
            unreachable!("root-motion commands return a pending physical step after admission")
        }
        CommandPayload::WorldRoutine(_) => {
            let routine = world_routine.ok_or(RuntimeFatalError::WorldRoutineInternalInvariant)?;
            let (event, delta) = routine.apply_stage_9(
                command,
                context.tick,
                context.phase_revision,
                candidate.command_id,
            )?;
            (staged.rpg.clone(), vec![event], delta, None)
        }
        CommandPayload::WorldPopulation(_) => {
            let population =
                world_population.ok_or(RuntimeFatalError::WorldPopulationInternalInvariant)?;
            let (event, delta) = population.apply_stage_9(
                command,
                context.tick,
                context.phase_revision,
                candidate.command_id,
            )?;
            (staged.rpg.clone(), vec![event], delta, None)
        }
        CommandPayload::WorldActivity(_) => {
            let activity =
                world_activity.ok_or(RuntimeFatalError::WorldActivityInternalInvariant)?;
            let (event, delta) = activity.apply_stage_9(
                command,
                context.tick,
                context.phase_revision,
                candidate.command_id,
                &staged.rpg,
            )?;
            (staged.rpg.clone(), vec![event], delta, None)
        }
        CommandPayload::WaterVolume(payload) => {
            // ADR-100: the authoritative water table lives in the physics
            // owner checkpoint and changes only here.
            let current = staged.physics.water_volumes().clone();
            match current.apply_command(payload, context.tick) {
                Ok((next, changed)) => {
                    let delta = next.canonical_record()?;
                    staged.physics.set_water_volumes(next);
                    let event = DomainEvent::water_volume(
                        context.tick,
                        context.phase,
                        candidate.command_id,
                        0,
                        changed,
                    )?;
                    (staged.rpg.clone(), vec![event], delta, None)
                }
                Err(rejection) => {
                    let code = match rejection {
                        WaterVolumeRejectionV1::UnknownVolume => RejectionCode::WaterVolumeUnknown,
                        WaterVolumeRejectionV1::RevisionStale => {
                            RejectionCode::WaterVolumeRevisionStale
                        }
                        WaterVolumeRejectionV1::LevelOutOfExtent => {
                            RejectionCode::WaterVolumeLevelOutOfExtent
                        }
                        WaterVolumeRejectionV1::RevisionExhausted => {
                            RejectionCode::WaterVolumeRevisionExhausted
                        }
                    };
                    return finalize_rejection(context, candidate, staged, code);
                }
            }
        }
        CommandPayload::WaterFlow(payload) => {
            // ADR-103: the flow network lives next to the water table in the
            // physics owner checkpoint and changes only here.
            let current = staged.physics.water_flow().clone();
            match current.apply_command(payload, context.tick) {
                Ok((next, changed)) => {
                    let delta = next.canonical_record()?;
                    staged.physics.set_water_flow(next);
                    let event = DomainEvent::water_flow(
                        context.tick,
                        context.phase,
                        candidate.command_id,
                        0,
                        changed,
                    )?;
                    (staged.rpg.clone(), vec![event], delta, None)
                }
                Err(rejection) => {
                    let code = match rejection {
                        WaterFlowRejectionV1::UnknownEdge => RejectionCode::WaterFlowEdgeUnknown,
                        WaterFlowRejectionV1::WrongKind => RejectionCode::WaterFlowEdgeKindMismatch,
                        WaterFlowRejectionV1::RevisionStale => {
                            RejectionCode::WaterFlowRevisionStale
                        }
                        WaterFlowRejectionV1::OpeningOutOfRange => {
                            RejectionCode::WaterFlowOpeningOutOfRange
                        }
                        WaterFlowRejectionV1::RateOutOfRange => {
                            RejectionCode::WaterFlowRateOutOfRange
                        }
                        WaterFlowRejectionV1::RevisionExhausted => {
                            RejectionCode::WaterFlowRevisionExhausted
                        }
                    };
                    return finalize_rejection(context, candidate, staged, code);
                }
            }
        }
        CommandPayload::AgentCognition(_) => {
            let cognition =
                agent_cognition.ok_or(RuntimeFatalError::AgentCognitionInternalInvariant)?;
            let (event, delta) = cognition.apply_stage_9(
                command,
                context.tick,
                context.phase_revision,
                candidate.command_id,
            )?;
            (staged.rpg.clone(), vec![event], delta, None)
        }
    };
    for event in &events {
        let insert = staged
            .ledger_delta
            .stage_event_identity(&staged.ledger, event)?;
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
    let receipt =
        command_receipt(
            context,
            staged.ledger.streams.get(&command.stream_id).ok_or(
                RuntimeFatalError::LedgerCorrupt(CommandLedgerError::StreamKeyMismatch),
            )?,
            &candidate,
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
    let body_hash = candidate.body_hash;
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
        .ledger_delta
        .identity_binding(&staged.ledger.identity_index, &candidate.command_id)
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
            &candidate,
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
