use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use next_contracts::canonical::{CanonicalDecodeLimits, sha256};
use next_contracts::command::{
    COMMAND_ENVELOPE_SCHEMA_VERSION, CommandPhase, DomainEvent, IssuerPrincipal, WorldCommand,
};
use next_contracts::identity::{
    CommandStreamRegistryV1, PrincipalRegistryV1, RuntimeDeterminismProfileV1,
};
use next_contracts::ids::{
    CommandBodyHash, CommandId, CommandStreamId, command_body_hash_from_bytes,
};
use next_contracts::input::{IngressCheckpointV1, PlayerControllerRegistryV1};
use next_contracts::ledger::{CommandBodyArchiveV1, CommandLedgerError, CommandLedgerV2};
use next_contracts::physics::{
    AcceptedLocomotionIntentV2, ClosedPhysicsContactBatchV1, PhysicsStepInputV2,
};
use next_contracts::rpg::{RpgPhysicalContactFactV1, RpgRuntimeBindingsV1};
use next_physics_api::PhysicsWorldHost;
use next_rpg::RpgState;

use crate::authority::AuthorityRegistry;
use crate::registry::CommandKindRegistry;

use super::agent_cognition::AgentCognitionStageContextV1;
use super::error::RuntimeFatalError;
use super::result::{
    CommandOrderKey, CommittedRpgPlanTraceV1, OrderedResult, RejectionCode, StageTraceEntry,
    TransactionStage,
};
use super::world_activity::WorldActivityStageContextV1;
use super::world_population::WorldPopulationStageContextV1;
use super::world_routine::WorldRoutineStageContextV1;

mod execution;
mod ledger;
mod physics_step;
mod transaction_delta;

use execution::execute_candidate;
use ledger::handle_collision;
use physics_step::finish_physical_step;
use transaction_delta::CommandLedgerTransactionDelta;
pub(crate) use transaction_delta::PreparedCommandLedgerTransaction;

type CollisionKey = (CommandStreamId, IssuerPrincipal, u64);

#[derive(Clone, Debug, Eq, PartialEq)]
struct ValidatedCommand {
    command: WorldCommand,
    command_id: CommandId,
    body_hash: CommandBodyHash,
    canonical_bytes: Vec<u8>,
    order_key: CommandOrderKey,
    due: bool,
}

#[derive(Clone, Debug)]
struct QueuedCommand {
    command: WorldCommand,
    due: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ValidationSource {
    ExternalIngress,
    InternalOutcome,
}

#[derive(Debug)]
pub(super) struct PhaseExecution {
    pub(super) results: Vec<OrderedResult>,
    pub(super) events: Vec<DomainEvent>,
    pub(super) stage_trace: Vec<StageTraceEntry>,
    pub(super) physics_step_input: Option<PhysicsStepInputV2>,
    pub(super) contact_batch: Option<ClosedPhysicsContactBatchV1>,
    pub(super) rpg_plan_traces: Vec<CommittedRpgPlanTraceV1>,
}

#[derive(Clone, Copy)]
pub(super) struct PhaseContext<'a> {
    pub(super) registry: &'a CommandKindRegistry,
    pub(super) authority: &'a AuthorityRegistry,
    pub(super) principals: &'a PrincipalRegistryV1,
    pub(super) streams: &'a CommandStreamRegistryV1,
    pub(super) profile: &'a RuntimeDeterminismProfileV1,
    pub(super) rpg_bindings: &'a RpgRuntimeBindingsV1,
    pub(super) controllers: &'a PlayerControllerRegistryV1,
    pub(super) physical_contact_facts: &'a [RpgPhysicalContactFactV1],
    pub(super) gameplay_hz: u32,
    pub(super) source: ValidationSource,
    pub(super) tick: u64,
    pub(super) phase: CommandPhase,
    pub(super) phase_revision: u64,
}

#[derive(Clone, Copy)]
pub(super) struct WorldStreamingStageContext<'a> {
    pub(super) world: &'a next_world::WorldStreamerV1,
    pub(super) publication: &'a next_world::PreparedWorldStreamingPublicationV1,
}

pub(super) struct StagedAuthoritativeState {
    pub(super) ledger: CommandLedgerV2,
    pub(super) archive: CommandBodyArchiveV1,
    pub(super) ledger_delta: CommandLedgerTransactionDelta,
    pub(super) event_count: u64,
    pub(super) revision: u64,
    pub(super) rpg: RpgState,
    pub(super) physics: PhysicsWorldHost,
    pub(super) ingress: IngressCheckpointV1,
}

enum CandidateExecution {
    Result(OrderedResult, ExecutionTrace),
    Committed(
        OrderedResult,
        Vec<DomainEvent>,
        Option<CommittedRpgPlanTraceV1>,
    ),
    PhysicalPending(Box<PhysicalPending>),
}

struct PhysicalPending {
    candidate: ValidatedCommand,
    intent: AcceptedLocomotionIntentV2,
}

#[derive(Clone, Copy)]
enum ExecutionTrace {
    Rejected,
    Deduplicated,
    Reserved,
}

struct PhysicalStepExecution {
    command_results: Vec<(OrderedResult, Option<DomainEvent>)>,
    step_input: PhysicsStepInputV2,
    contact_batch: ClosedPhysicsContactBatchV1,
}

#[allow(
    clippy::too_many_arguments,
    reason = "phase execution threads each optional staged authoritative owner explicitly"
)]
pub(super) fn process_phase(
    context: PhaseContext<'_>,
    commands: Vec<WorldCommand>,
    staged: &mut StagedAuthoritativeState,
    world_streaming: Option<WorldStreamingStageContext<'_>>,
    mut world_routine: Option<&mut WorldRoutineStageContextV1>,
    mut world_population: Option<&mut WorldPopulationStageContextV1>,
    mut world_activity: Option<&mut WorldActivityStageContextV1>,
    mut agent_cognition: Option<&mut AgentCognitionStageContextV1>,
) -> Result<PhaseExecution, RuntimeFatalError> {
    let mut queued = due_commands(context, staged)?;
    queued.extend(commands.into_iter().map(|command| QueuedCommand {
        command,
        due: false,
    }));
    let received = count(queued.len())?;
    queued.sort_by(|left, right| left.command.cmp(&right.command));

    let mut results = Vec::new();
    let mut admission_rejected = 0_u64;
    let mut deduplicated = 0_u64;
    let mut valid = Vec::new();
    for queued in queued {
        let order_key = CommandOrderKey::from_command(&queued.command, context.registry);
        let command_id = queued.command.compute_command_id().unwrap_or_default();
        let sequence = queued.command.sequence;
        match validate_command(context, queued) {
            Ok(command) => valid.push(command),
            Err(code) => {
                admission_rejected = checked_inc(admission_rejected)?;
                results.push(OrderedResult::rejected(
                    order_key, command_id, sequence, code,
                ));
            }
        }
    }

    let mut groups: BTreeMap<CollisionKey, Vec<ValidatedCommand>> = BTreeMap::new();
    for command in valid {
        groups
            .entry((
                command.command.stream_id,
                command.command.issuer.clone(),
                command.command.sequence,
            ))
            .or_default()
            .push(command);
    }

    let mut candidates = Vec::new();
    for mut group in groups.into_values() {
        group.sort_by(compare_validated_commands);
        let mut unique: BTreeMap<Vec<u8>, ValidatedCommand> = BTreeMap::new();
        for candidate in group {
            match unique.get_mut(&candidate.canonical_bytes) {
                Some(existing) => {
                    existing.due |= candidate.due;
                    deduplicated = checked_inc(deduplicated)?;
                }
                None => {
                    unique.insert(candidate.canonical_bytes.clone(), candidate);
                }
            }
        }
        let group: Vec<_> = unique.into_values().collect();
        if group.len() > 1 {
            let collision_results = handle_collision(context, group, staged)?;
            admission_rejected = admission_rejected
                .checked_add(count(collision_results.len())?)
                .ok_or(RuntimeFatalError::TraceCountExhausted)?;
            results.extend(collision_results);
        } else if let Some(candidate) = group.into_iter().next() {
            candidates.push(candidate);
        }
    }
    candidates.sort_by(compare_execution_candidates);
    let admitted = count(candidates.len())?;

    let mut commit_rejected = 0_u64;
    let mut committed = 0_u64;
    let mut commit_deduplicated = 0_u64;
    let mut events = Vec::new();
    let mut physical_pending = Vec::new();
    let mut physical_bodies = BTreeSet::new();
    let mut physics_step_input = None;
    let mut contact_batch = None;
    let mut rpg_plan_traces = Vec::new();
    for candidate in candidates {
        match execute_candidate(
            context,
            candidate,
            staged,
            &mut physical_bodies,
            world_routine.as_deref_mut(),
            world_population.as_deref_mut(),
            world_activity.as_deref_mut(),
            agent_cognition.as_deref_mut(),
        )? {
            CandidateExecution::Result(result, trace) => {
                match trace {
                    ExecutionTrace::Rejected => commit_rejected = checked_inc(commit_rejected)?,
                    ExecutionTrace::Deduplicated => {
                        commit_deduplicated = checked_inc(commit_deduplicated)?
                    }
                    ExecutionTrace::Reserved => {}
                }
                results.push(result);
            }
            CandidateExecution::Committed(result, committed_events, plan_trace) => {
                committed = checked_inc(committed)?;
                results.push(result);
                events.extend(committed_events);
                if let Some(plan_trace) = plan_trace {
                    rpg_plan_traces.push(plan_trace);
                }
            }
            CandidateExecution::PhysicalPending(pending) => physical_pending.push(*pending),
        }
    }
    if context.source == ValidationSource::ExternalIngress {
        if let Some(world_streaming) = world_streaming {
            world_streaming
                .world
                .validate_prepared_stage(world_streaming.publication, context.tick)?;
        }
        if let Some(world_routine) = world_routine.as_deref_mut() {
            // The stage-6 envelope binds the exact stage-9 Runtime revision.
            // The mandatory stage-8 physics step advances the canonical
            // physics snapshot (including its tick) and therefore contributes
            // exactly one authoritative revision before Outcome admission.
            let outcome_phase_revision = staged
                .revision
                .checked_add(1)
                .ok_or(RuntimeFatalError::RevisionExhausted)?;
            world_routine.produce_stage_6(context.tick, outcome_phase_revision)?;
        }
        if let Some(world_population) = world_population.as_deref_mut() {
            let outcome_phase_revision = staged
                .revision
                .checked_add(1)
                .ok_or(RuntimeFatalError::RevisionExhausted)?;
            world_population.produce_stage_6(context.tick, outcome_phase_revision)?;
        }
        if let Some(agent_cognition) = agent_cognition.as_deref_mut() {
            let outcome_phase_revision = staged
                .revision
                .checked_add(1)
                .ok_or(RuntimeFatalError::RevisionExhausted)?;
            agent_cognition.produce_stage_7(context.tick, outcome_phase_revision, &staged.rpg)?;
        }
        if let Some(world_activity) = world_activity.as_deref_mut() {
            let outcome_phase_revision = staged
                .revision
                .checked_add(1)
                .ok_or(RuntimeFatalError::RevisionExhausted)?;
            world_activity.produce_stage_7(context.tick, outcome_phase_revision, &staged.rpg)?;
        }
        let physical_execution = finish_physical_step(context, physical_pending, staged)?;
        for (result, event) in physical_execution.command_results {
            committed = checked_inc(committed)?;
            results.push(result);
            if let Some(event) = event {
                events.push(event);
            }
        }
        physics_step_input = Some(physical_execution.step_input);
        contact_batch = Some(physical_execution.contact_batch);
    } else if !physical_pending.is_empty() {
        return Err(RuntimeFatalError::PhysicalOutcomeInvariant);
    }

    results.sort();
    let admission_stage = match context.source {
        ValidationSource::ExternalIngress => TransactionStage::IngressAdmission,
        ValidationSource::InternalOutcome => TransactionStage::OutcomeAdmission,
    };
    let commit_stage = match context.source {
        ValidationSource::ExternalIngress => TransactionStage::IngressCommit,
        ValidationSource::InternalOutcome => TransactionStage::OutcomeCommit,
    };
    let mut stage_trace = vec![
        StageTraceEntry {
            stage: admission_stage,
            received,
            accepted: admitted,
            rejected: admission_rejected,
            committed: 0,
            deduplicated,
        },
        StageTraceEntry {
            stage: commit_stage,
            received: admitted,
            accepted: admitted
                .checked_sub(commit_rejected)
                .and_then(|value| value.checked_sub(commit_deduplicated))
                .ok_or(RuntimeFatalError::TraceCountExhausted)?,
            rejected: commit_rejected,
            committed,
            deduplicated: commit_deduplicated,
        },
    ];
    if world_streaming.is_some()
        || world_routine.is_some()
        || world_population.is_some()
        || world_activity.is_some()
    {
        stage_trace.push(StageTraceEntry {
            stage: TransactionStage::WorldStreamingCommit,
            received: 1,
            accepted: 1,
            rejected: 0,
            committed: 1,
            deduplicated: 0,
        });
    }
    if agent_cognition.is_some() {
        stage_trace.push(StageTraceEntry {
            stage: TransactionStage::AgentPlanning,
            received: 1,
            accepted: 1,
            rejected: 0,
            committed: 1,
            deduplicated: 0,
        });
    }
    Ok(PhaseExecution {
        results,
        events,
        physics_step_input,
        contact_batch,
        rpg_plan_traces,
        stage_trace,
    })
}

fn due_commands(
    context: PhaseContext<'_>,
    staged: &StagedAuthoritativeState,
) -> Result<Vec<QueuedCommand>, RuntimeFatalError> {
    let mut commands = Vec::new();
    for stream in staged.ledger.streams.values() {
        for reservation in stream.pending.values() {
            if reservation.target_tick < context.tick {
                return Err(RuntimeFatalError::LedgerCorrupt(
                    CommandLedgerError::TargetTickRegression,
                ));
            }
            if reservation.target_tick == context.tick && reservation.phase == context.phase {
                let bytes = staged
                    .ledger_delta
                    .archive_bytes(&staged.archive, &reservation.canonical_body_ref)
                    .ok_or(RuntimeFatalError::LedgerCorrupt(
                        CommandLedgerError::BodyReferenceMissing,
                    ))?;
                let command =
                    WorldCommand::from_canonical_bytes(bytes, CanonicalDecodeLimits::default())
                        .map_err(CommandLedgerError::from)
                        .map_err(RuntimeFatalError::LedgerCorrupt)?;
                commands.push(QueuedCommand { command, due: true });
            }
        }
    }
    Ok(commands)
}

fn validate_command(
    context: PhaseContext<'_>,
    queued: QueuedCommand,
) -> Result<ValidatedCommand, RejectionCode> {
    let command = queued.command;
    let descriptor = context
        .registry
        .descriptor(&command.payload_schema_id, command.payload_schema_version)
        .filter(|descriptor| descriptor.accepts_payload(&command.payload))
        .ok_or(RejectionCode::SchemaMismatch)?;
    if command.envelope_schema_version != COMMAND_ENVELOPE_SCHEMA_VERSION {
        return Err(RejectionCode::SchemaMismatch);
    }
    let canonical_bytes = command
        .canonical_bytes()
        .map_err(|_| RejectionCode::CanonicalCommandInvalid)?;
    let command_id = command
        .compute_command_id()
        .map_err(|_| RejectionCode::CanonicalCommandInvalid)?;
    let body_hash = command_body_hash_from_bytes(sha256(&canonical_bytes));
    if command
        .claimed_command_id
        .is_some_and(|claimed| claimed != command_id)
    {
        return Err(RejectionCode::CommandIdMismatch);
    }
    match context.source {
        ValidationSource::InternalOutcome
            if !matches!(command.issuer, IssuerPrincipal::InternalSystem(_)) =>
        {
            return Err(RejectionCode::InternalOutcomeIssuerRequired);
        }
        _ => {}
    }
    if !context.principals.is_active(&command.issuer) {
        return Err(RejectionCode::IssuerUnauthenticated);
    }
    let Some((stream_key, _)) = context.streams.binding(command.stream_id) else {
        return Err(RejectionCode::CommandStreamUnbound);
    };
    if stream_key.principal != command.issuer {
        return Err(RejectionCode::CommandStreamUnbound);
    }
    let grants = context
        .authority
        .grants(&command.issuer)
        .ok_or(RejectionCode::IssuerUnauthenticated)?;
    let declared: BTreeSet<_> = command
        .capability_claims
        .iter()
        .map(|claim| claim.capability_id.clone())
        .collect();
    if descriptor.required_capabilities().iter().any(|required| {
        required.scope_hash.is_some() || !declared.contains(&required.capability_id)
    }) {
        return Err(RejectionCode::CapabilityRequired);
    }
    if declared
        .iter()
        .any(|capability| !grants.contains(capability))
    {
        return Err(RejectionCode::CapabilityDenied);
    }
    Ok(ValidatedCommand {
        order_key: CommandOrderKey::new(&command, descriptor.priority_class()),
        command,
        command_id,
        body_hash,
        canonical_bytes,
        due: queued.due,
    })
}

fn compare_validated_commands(left: &ValidatedCommand, right: &ValidatedCommand) -> Ordering {
    left.order_key.cmp(&right.order_key)
}

fn compare_execution_candidates(left: &ValidatedCommand, right: &ValidatedCommand) -> Ordering {
    if left.command.stream_id == right.command.stream_id {
        (left.command.sequence, &left.order_key).cmp(&(right.command.sequence, &right.order_key))
    } else {
        (
            &left.order_key,
            left.command.stream_id,
            left.command.sequence,
        )
            .cmp(&(
                &right.order_key,
                right.command.stream_id,
                right.command.sequence,
            ))
    }
}

pub(super) fn count(value: usize) -> Result<u64, RuntimeFatalError> {
    u64::try_from(value).map_err(|_| RuntimeFatalError::TraceCountExhausted)
}

fn checked_inc(value: u64) -> Result<u64, RuntimeFatalError> {
    value
        .checked_add(1)
        .ok_or(RuntimeFatalError::TraceCountExhausted)
}
