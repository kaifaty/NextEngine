use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::{
    CanonicalError, CommandId, CommandLedgerSnapshot, CommandPhase, CommandStreamId, DomainEvent,
    IssuerPrincipal, RuntimeSnapshot, WorldCommand,
};

use crate::authority::AuthorityRegistry;
use crate::outcome::{
    NoOutcomes, OutcomeCollectionError, OutcomeContext, OutcomeProvider, OutcomeSink,
};
use crate::registry::CommandKindRegistry;

type LedgerKey = (CommandStreamId, IssuerPrincipal);
type CollisionKey = (CommandStreamId, IssuerPrincipal, u64);

#[derive(Clone, Debug)]
pub struct RuntimeState {
    registry: CommandKindRegistry,
    authority: AuthorityRegistry,
    next_tick: u64,
    committed_event_count: u64,
    authoritative_revision: u64,
    ledgers: BTreeMap<LedgerKey, LedgerEntry>,
}

impl RuntimeState {
    #[must_use]
    pub fn new(authority: AuthorityRegistry) -> Self {
        Self {
            registry: CommandKindRegistry::core_v1(),
            authority,
            next_tick: 0,
            committed_event_count: 0,
            authoritative_revision: 0,
            ledgers: BTreeMap::new(),
        }
    }

    #[must_use]
    pub const fn next_tick(&self) -> u64 {
        self.next_tick
    }

    #[must_use]
    pub const fn authoritative_revision(&self) -> u64 {
        self.authoritative_revision
    }

    #[must_use]
    pub fn command_kind_registry(&self) -> &CommandKindRegistry {
        &self.registry
    }

    #[must_use]
    pub fn snapshot(&self) -> RuntimeSnapshot {
        snapshot_from(
            self.next_tick,
            self.committed_event_count,
            self.authoritative_revision,
            &self.ledgers,
        )
    }

    pub fn run_tick(
        &mut self,
        commands: impl IntoIterator<Item = WorldCommand>,
    ) -> Result<TickReport, RuntimeFatalError> {
        self.run_tick_with_outcomes(commands, &mut NoOutcomes)
    }

    pub fn run_tick_with_outcomes(
        &mut self,
        commands: impl IntoIterator<Item = WorldCommand>,
        outcome_provider: &mut impl OutcomeProvider,
    ) -> Result<TickReport, RuntimeFatalError> {
        let following_tick = self
            .next_tick
            .checked_add(1)
            .ok_or(RuntimeFatalError::TickExhausted)?;
        let tick = self.next_tick;
        let mut staged_ledgers = self.ledgers.clone();
        let mut staged_event_count = self.committed_event_count;
        let mut staged_revision = self.authoritative_revision;

        let ingress_revision = staged_revision;
        let ingress = process_phase(
            PhaseContext {
                registry: &self.registry,
                authority: &self.authority,
                source: ValidationSource::ExternalIngress,
                tick,
                phase_revision: ingress_revision,
            },
            commands.into_iter().collect(),
            StagedAuthoritativeState {
                ledgers: &mut staged_ledgers,
                event_count: &mut staged_event_count,
                revision: &mut staged_revision,
            },
        )?;

        let mut outcome_sink = OutcomeSink::new();
        outcome_provider
            .collect(
                OutcomeContext {
                    tick,
                    authoritative_revision: staged_revision,
                    ingress_events: &ingress.events,
                },
                &mut outcome_sink,
            )
            .map_err(RuntimeFatalError::OutcomeCollection)?;
        let proposals = outcome_sink.into_proposals();
        let proposal_count = count(proposals.len())?;
        let outcome_commands = proposals
            .into_iter()
            .map(|proposal| {
                proposal
                    .into_command(tick)
                    .map_err(RuntimeFatalError::InternalCanonicalization)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let outcome_revision = staged_revision;
        let outcome = process_phase(
            PhaseContext {
                registry: &self.registry,
                authority: &self.authority,
                source: ValidationSource::InternalOutcome,
                tick,
                phase_revision: outcome_revision,
            },
            outcome_commands,
            StagedAuthoritativeState {
                ledgers: &mut staged_ledgers,
                event_count: &mut staged_event_count,
                revision: &mut staged_revision,
            },
        )?;

        let mut ordered_results = ingress.results;
        ordered_results.extend(outcome.results);
        ordered_results.sort();
        let results = ordered_results
            .into_iter()
            .map(|result| result.result)
            .collect();

        let mut events = ingress.events;
        events.extend(outcome.events);
        let snapshot = snapshot_from(
            following_tick,
            staged_event_count,
            staged_revision,
            &staged_ledgers,
        );

        let mut stage_trace = ingress.stage_trace;
        stage_trace.push(StageTraceEntry {
            stage: TransactionStage::OutcomeCollection,
            received: proposal_count,
            accepted: proposal_count,
            rejected: 0,
            committed: 0,
            deduplicated: 0,
        });
        stage_trace.extend(outcome.stage_trace);
        stage_trace.push(StageTraceEntry {
            stage: TransactionStage::SnapshotPublication,
            received: 1,
            accepted: 1,
            rejected: 0,
            committed: 1,
            deduplicated: 0,
        });

        self.next_tick = following_tick;
        self.committed_event_count = staged_event_count;
        self.authoritative_revision = staged_revision;
        self.ledgers = staged_ledgers;

        Ok(TickReport {
            tick,
            results,
            events,
            stage_trace,
            snapshot,
        })
    }
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self::new(AuthorityRegistry::new())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LedgerEntry {
    last_sequence: u64,
    command_id: CommandId,
    canonical_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ValidatedCommand {
    command: WorldCommand,
    canonical_bytes: Vec<u8>,
    order_key: CommandOrderKey,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ValidationSource {
    ExternalIngress,
    InternalOutcome,
}

#[derive(Debug)]
struct PhaseExecution {
    results: Vec<OrderedResult>,
    events: Vec<DomainEvent>,
    stage_trace: Vec<StageTraceEntry>,
}

#[derive(Clone, Copy)]
struct PhaseContext<'a> {
    registry: &'a CommandKindRegistry,
    authority: &'a AuthorityRegistry,
    source: ValidationSource,
    tick: u64,
    phase_revision: u64,
}

struct StagedAuthoritativeState<'a> {
    ledgers: &'a mut BTreeMap<LedgerKey, LedgerEntry>,
    event_count: &'a mut u64,
    revision: &'a mut u64,
}

fn process_phase(
    context: PhaseContext<'_>,
    mut commands: Vec<WorldCommand>,
    staged: StagedAuthoritativeState<'_>,
) -> Result<PhaseExecution, RuntimeFatalError> {
    let received = count(commands.len())?;
    commands.sort();
    commands.dedup();
    let mut deduplicated = received
        .checked_sub(count(commands.len())?)
        .ok_or(RuntimeFatalError::TraceCountExhausted)?;

    let mut results = Vec::new();
    let mut collision_groups: BTreeMap<CollisionKey, Vec<WorldCommand>> = BTreeMap::new();
    for command in commands {
        collision_groups
            .entry((command.stream_id, command.issuer.clone(), command.sequence))
            .or_default()
            .push(command);
    }

    let mut candidates = Vec::new();
    let mut admission_rejected = 0_u64;
    for mut group in collision_groups.into_values() {
        group.sort();
        let canonical_group: Vec<_> = group.iter().map(WorldCommand::canonical_bytes).collect();
        let exact_canonical_equivalence = canonical_group
            .first()
            .and_then(|bytes| bytes.as_ref().ok())
            .is_some_and(|first_bytes| {
                canonical_group
                    .iter()
                    .all(|bytes| bytes.as_ref().is_ok_and(|bytes| bytes == first_bytes))
                    && group
                        .iter()
                        .all(|command| command.command_id == group[0].command_id)
            });
        if group.len() > 1 && !exact_canonical_equivalence {
            admission_rejected = admission_rejected
                .checked_add(count(group.len())?)
                .ok_or(RuntimeFatalError::TraceCountExhausted)?;
            results.extend(group.into_iter().map(|command| {
                OrderedResult::rejected(
                    CommandOrderKey::from_command(&command, context.registry),
                    command.command_id,
                    command.sequence,
                    RejectionCode::CommandSequenceCollision,
                )
            }));
        } else if let Some(command) = group.into_iter().next() {
            if exact_canonical_equivalence {
                deduplicated = deduplicated
                    .checked_add(
                        count(canonical_group.len())?
                            .checked_sub(1)
                            .ok_or(RuntimeFatalError::TraceCountExhausted)?,
                    )
                    .ok_or(RuntimeFatalError::TraceCountExhausted)?;
            }
            let order_key = CommandOrderKey::from_command(&command, context.registry);
            let command_id = command.command_id;
            let sequence = command.sequence;
            let canonical_bytes = canonical_group
                .into_iter()
                .next()
                .expect("nonempty collision group has canonical result");
            match validate_command(
                context.registry,
                context.authority,
                context.source,
                command,
                canonical_bytes,
            ) {
                Ok(candidate) => candidates.push(candidate),
                Err(code) => {
                    admission_rejected = admission_rejected
                        .checked_add(1)
                        .ok_or(RuntimeFatalError::TraceCountExhausted)?;
                    results.push(OrderedResult::rejected(
                        order_key, command_id, sequence, code,
                    ));
                }
            }
        }
    }
    candidates.sort_by(compare_validated_commands);
    let admitted = count(candidates.len())?;
    let admission_stage = match context.source {
        ValidationSource::ExternalIngress => TransactionStage::IngressAdmission,
        ValidationSource::InternalOutcome => TransactionStage::OutcomeAdmission,
    };
    let commit_stage = match context.source {
        ValidationSource::ExternalIngress => TransactionStage::IngressCommit,
        ValidationSource::InternalOutcome => TransactionStage::OutcomeCommit,
    };

    let mut commit_rejected = 0_u64;
    let mut committed = 0_u64;
    let mut committed_retry_deduplicated = 0_u64;
    let mut events = Vec::new();
    for candidate in candidates {
        let command = candidate.command;
        let ledger_key = (command.stream_id, command.issuer.clone());
        if let Some(entry) = staged.ledgers.get(&ledger_key) {
            if command.sequence == entry.last_sequence
                && command.command_id == entry.command_id
                && candidate.canonical_bytes == entry.canonical_bytes
            {
                committed_retry_deduplicated = committed_retry_deduplicated
                    .checked_add(1)
                    .ok_or(RuntimeFatalError::TraceCountExhausted)?;
                results.push(OrderedResult::deduplicated(
                    candidate.order_key,
                    command.command_id,
                    command.sequence,
                ));
                continue;
            }
            if entry.last_sequence == u64::MAX {
                commit_rejected = commit_rejected
                    .checked_add(1)
                    .ok_or(RuntimeFatalError::TraceCountExhausted)?;
                results.push(OrderedResult::rejected(
                    candidate.order_key,
                    command.command_id,
                    command.sequence,
                    RejectionCode::CommandSequenceExhausted,
                ));
                continue;
            }
            if command.sequence <= entry.last_sequence {
                commit_rejected = commit_rejected
                    .checked_add(1)
                    .ok_or(RuntimeFatalError::TraceCountExhausted)?;
                results.push(OrderedResult::rejected(
                    candidate.order_key,
                    command.command_id,
                    command.sequence,
                    RejectionCode::CommandSequenceReuse,
                ));
                continue;
            }
        }

        if command.target_tick != context.tick {
            commit_rejected = commit_rejected
                .checked_add(1)
                .ok_or(RuntimeFatalError::TraceCountExhausted)?;
            results.push(OrderedResult::rejected(
                candidate.order_key,
                command.command_id,
                command.sequence,
                RejectionCode::TargetTickMismatch,
            ));
            continue;
        }
        if command
            .precondition_revision
            .is_some_and(|revision| revision != context.phase_revision)
        {
            commit_rejected = commit_rejected
                .checked_add(1)
                .ok_or(RuntimeFatalError::TraceCountExhausted)?;
            results.push(OrderedResult::rejected(
                candidate.order_key,
                command.command_id,
                command.sequence,
                RejectionCode::PreconditionFailed,
            ));
            continue;
        }

        let event =
            DomainEvent::command_committed(context.tick, command.command_id, command.sequence)
                .map_err(RuntimeFatalError::InternalCanonicalization)?;
        *staged.event_count = staged
            .event_count
            .checked_add(1)
            .ok_or(RuntimeFatalError::EventCountExhausted)?;
        *staged.revision = staged
            .revision
            .checked_add(1)
            .ok_or(RuntimeFatalError::RevisionExhausted)?;
        staged.ledgers.insert(
            ledger_key,
            LedgerEntry {
                last_sequence: command.sequence,
                command_id: command.command_id,
                canonical_bytes: candidate.canonical_bytes,
            },
        );
        committed = committed
            .checked_add(1)
            .ok_or(RuntimeFatalError::TraceCountExhausted)?;
        results.push(OrderedResult::committed(
            candidate.order_key,
            command.command_id,
            command.sequence,
        ));
        events.push(event);
    }

    results.sort();
    Ok(PhaseExecution {
        results,
        events,
        stage_trace: vec![
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
                    .and_then(|count| count.checked_sub(committed_retry_deduplicated))
                    .ok_or(RuntimeFatalError::TraceCountExhausted)?,
                rejected: commit_rejected,
                committed,
                deduplicated: committed_retry_deduplicated,
            },
        ],
    })
}

fn validate_command(
    registry: &CommandKindRegistry,
    authority: &AuthorityRegistry,
    source: ValidationSource,
    command: WorldCommand,
    canonical_bytes: Result<Vec<u8>, CanonicalError>,
) -> Result<ValidatedCommand, RejectionCode> {
    let descriptor = registry
        .descriptor(&command.schema_id, command.schema_version)
        .filter(|descriptor| descriptor.accepts_payload(&command.payload))
        .ok_or(RejectionCode::SchemaMismatch)?;
    if !descriptor.allows_phase(command.phase) {
        return Err(RejectionCode::CommandPhaseForbidden);
    }
    match source {
        ValidationSource::ExternalIngress if command.phase != CommandPhase::Ingress => {
            return Err(RejectionCode::PublicOutcomeForbidden);
        }
        ValidationSource::InternalOutcome
            if command.phase != CommandPhase::Outcome
                || !matches!(command.issuer, IssuerPrincipal::InternalSystem(_)) =>
        {
            return Err(RejectionCode::InternalOutcomeIssuerRequired);
        }
        _ => {}
    }

    let canonical_bytes = canonical_bytes.map_err(|_| RejectionCode::CanonicalCommandInvalid)?;
    let expected_id = command
        .compute_command_id()
        .map_err(|_| RejectionCode::CanonicalCommandInvalid)?;
    if command.command_id != expected_id {
        return Err(RejectionCode::CommandIdMismatch);
    }

    let grants = authority
        .grants(&command.issuer)
        .ok_or(RejectionCode::IssuerUnauthenticated)?;
    let declared: BTreeSet<_> = command.declared_capabilities.iter().cloned().collect();
    for required in descriptor.required_capabilities() {
        if !declared.contains(required) {
            return Err(RejectionCode::CapabilityRequired);
        }
    }
    if declared
        .iter()
        .any(|capability| !grants.contains(capability))
    {
        return Err(RejectionCode::CapabilityDenied);
    }
    if command.target.is_some() {
        return Err(RejectionCode::TargetNotAllowed);
    }
    Ok(ValidatedCommand {
        order_key: CommandOrderKey::new(&command, descriptor.priority_class()),
        command,
        canonical_bytes,
    })
}

fn compare_validated_commands(left: &ValidatedCommand, right: &ValidatedCommand) -> Ordering {
    left.order_key.cmp(&right.order_key)
}

fn snapshot_from(
    next_tick: u64,
    committed_event_count: u64,
    authoritative_revision: u64,
    ledgers: &BTreeMap<LedgerKey, LedgerEntry>,
) -> RuntimeSnapshot {
    RuntimeSnapshot {
        next_tick,
        committed_event_count,
        authoritative_revision,
        command_ledgers: ledgers
            .iter()
            .map(|((stream_id, issuer), entry)| CommandLedgerSnapshot {
                stream_id: *stream_id,
                issuer: issuer.clone(),
                last_sequence: entry.last_sequence,
                command_id: entry.command_id,
            })
            .collect(),
    }
}

fn count(value: usize) -> Result<u64, RuntimeFatalError> {
    u64::try_from(value).map_err(|_| RuntimeFatalError::TraceCountExhausted)
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct CommandOrderKey {
    target_tick: u64,
    phase: CommandPhase,
    priority_class: u16,
    issuer_tag: u8,
    issuer_id_bytes: Vec<u8>,
    sequence: u64,
    command_id: CommandId,
}

impl CommandOrderKey {
    fn new(command: &WorldCommand, priority_class: u16) -> Self {
        Self {
            target_tick: command.target_tick,
            phase: command.phase,
            priority_class,
            issuer_tag: command.issuer.tag(),
            issuer_id_bytes: command.issuer.identifier_bytes().to_vec(),
            sequence: command.sequence,
            command_id: command.command_id,
        }
    }

    fn from_command(command: &WorldCommand, registry: &CommandKindRegistry) -> Self {
        let priority_class = registry
            .descriptor(&command.schema_id, command.schema_version)
            .filter(|descriptor| descriptor.accepts_payload(&command.payload))
            .map_or(u16::MAX, |descriptor| descriptor.priority_class());
        Self::new(command, priority_class)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RejectionCode {
    SchemaMismatch,
    CanonicalCommandInvalid,
    CommandIdMismatch,
    CommandPhaseForbidden,
    PublicOutcomeForbidden,
    InternalOutcomeIssuerRequired,
    IssuerUnauthenticated,
    CapabilityRequired,
    CapabilityDenied,
    PreconditionFailed,
    TargetNotAllowed,
    TargetTickMismatch,
    CommandSequenceCollision,
    CommandSequenceReuse,
    CommandSequenceExhausted,
}

impl RejectionCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SchemaMismatch => "COMMAND_SCHEMA_MISMATCH",
            Self::CanonicalCommandInvalid => "CANONICAL_COMMAND_INVALID",
            Self::CommandIdMismatch => "COMMAND_ID_MISMATCH",
            Self::CommandPhaseForbidden => "COMMAND_PHASE_FORBIDDEN",
            Self::PublicOutcomeForbidden => "PUBLIC_OUTCOME_FORBIDDEN",
            Self::InternalOutcomeIssuerRequired => "INTERNAL_OUTCOME_ISSUER_REQUIRED",
            Self::IssuerUnauthenticated => "ISSUER_UNAUTHENTICATED",
            Self::CapabilityRequired => "CAPABILITY_REQUIRED",
            Self::CapabilityDenied => "CAPABILITY_DENIED",
            Self::PreconditionFailed => "PRECONDITION_FAILED",
            Self::TargetNotAllowed => "COMMAND_TARGET_NOT_ALLOWED",
            Self::TargetTickMismatch => "TARGET_TICK_MISMATCH",
            Self::CommandSequenceCollision => "COMMAND_SEQUENCE_COLLISION",
            Self::CommandSequenceReuse => "COMMAND_SEQUENCE_REUSE",
            Self::CommandSequenceExhausted => "COMMAND_SEQUENCE_EXHAUSTED",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommandDisposition {
    Committed,
    Deduplicated,
    Rejected(RejectionCode),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandResult {
    pub command_id: CommandId,
    pub sequence: u64,
    pub disposition: CommandDisposition,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum TransactionStage {
    IngressAdmission,
    IngressCommit,
    OutcomeCollection,
    OutcomeAdmission,
    OutcomeCommit,
    SnapshotPublication,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StageTraceEntry {
    pub stage: TransactionStage,
    pub received: u64,
    pub accepted: u64,
    pub rejected: u64,
    pub committed: u64,
    pub deduplicated: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TickReport {
    pub tick: u64,
    pub results: Vec<CommandResult>,
    pub events: Vec<DomainEvent>,
    pub stage_trace: Vec<StageTraceEntry>,
    pub snapshot: RuntimeSnapshot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct OrderedResult {
    order_key: CommandOrderKey,
    result: CommandResult,
}

impl OrderedResult {
    fn committed(order_key: CommandOrderKey, command_id: CommandId, sequence: u64) -> Self {
        Self {
            order_key,
            result: CommandResult {
                command_id,
                sequence,
                disposition: CommandDisposition::Committed,
            },
        }
    }

    fn deduplicated(order_key: CommandOrderKey, command_id: CommandId, sequence: u64) -> Self {
        Self {
            order_key,
            result: CommandResult {
                command_id,
                sequence,
                disposition: CommandDisposition::Deduplicated,
            },
        }
    }

    fn rejected(
        order_key: CommandOrderKey,
        command_id: CommandId,
        sequence: u64,
        code: RejectionCode,
    ) -> Self {
        Self {
            order_key,
            result: CommandResult {
                command_id,
                sequence,
                disposition: CommandDisposition::Rejected(code),
            },
        }
    }
}

impl Ord for OrderedResult {
    fn cmp(&self, other: &Self) -> Ordering {
        self.order_key.cmp(&other.order_key)
    }
}

impl PartialOrd for OrderedResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeFatalError {
    TickExhausted,
    EventCountExhausted,
    RevisionExhausted,
    TraceCountExhausted,
    OutcomeCollection(OutcomeCollectionError),
    InternalCanonicalization(CanonicalError),
}

impl RuntimeFatalError {
    #[must_use]
    pub const fn stable_code(self) -> &'static str {
        match self {
            Self::TickExhausted => "SIMULATION_TICK_EXHAUSTED",
            Self::EventCountExhausted => "DOMAIN_EVENT_COUNT_EXHAUSTED",
            Self::RevisionExhausted => "AUTHORITATIVE_REVISION_EXHAUSTED",
            Self::TraceCountExhausted => "STAGE_TRACE_COUNT_EXHAUSTED",
            Self::OutcomeCollection(error) => error.stable_code(),
            Self::InternalCanonicalization(_) => "INTERNAL_CANONICALIZATION_FAILED",
        }
    }
}

impl Display for RuntimeFatalError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InternalCanonicalization(error) => {
                write!(formatter, "{}: {error}", self.stable_code())
            }
            _ => formatter.write_str(self.stable_code()),
        }
    }
}

impl Error for RuntimeFatalError {}

#[cfg(test)]
mod tests {
    use next_contracts::{
        CapabilityId, CommandId, CommandPhase, CommandStreamId, IssuerPrincipal,
        NOOP_COMMAND_CAPABILITY_ID, PlayerPrincipalId, SchemaId, SystemId, WorldCommand,
    };

    use crate::authority::AuthorityRegistry;
    use crate::outcome::{
        OutcomeCollectionError, OutcomeContext, OutcomeProposal, OutcomeProvider, OutcomeSink,
    };

    use super::{
        CommandDisposition, RejectionCode, RuntimeFatalError, RuntimeState, StageTraceEntry,
        TransactionStage,
    };

    fn player(issuer: u8) -> IssuerPrincipal {
        IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([issuer; 16]))
    }

    fn noop_capability() -> CapabilityId {
        CapabilityId::new(NOOP_COMMAND_CAPABILITY_ID).expect("built-in capability is valid")
    }

    fn authority(principals: impl IntoIterator<Item = IssuerPrincipal>) -> AuthorityRegistry {
        let mut authority = AuthorityRegistry::new();
        for principal in principals {
            authority
                .register(principal, [noop_capability()])
                .expect("test principal is unique");
        }
        authority
    }

    fn command(stream: u8, issuer: u8, sequence: u64, target_tick: u64) -> WorldCommand {
        WorldCommand::noop(
            CommandStreamId::from_bytes([stream; 16]),
            player(issuer),
            sequence,
            target_tick,
        )
        .expect("test command is canonical")
    }

    fn runtime_for_players(players: impl IntoIterator<Item = u8>) -> RuntimeState {
        RuntimeState::new(authority(players.into_iter().map(player)))
    }

    #[test]
    fn arrival_permutations_produce_identical_reports() {
        let first = command(1, 2, 0, 0);
        let second = command(2, 1, 0, 0);
        let mut left = runtime_for_players([1, 2]);
        let mut right = runtime_for_players([1, 2]);

        let left_report = left
            .run_tick([first.clone(), second.clone()])
            .expect("tick should commit");
        let right_report = right.run_tick([second, first]).expect("tick should commit");

        assert_eq!(left_report, right_report);
        assert_eq!(left.snapshot(), right.snapshot());
    }

    #[test]
    fn exact_batch_duplicates_commit_once_and_trace_the_collapse() {
        let command = command(1, 1, 0, 0);
        let mut runtime = runtime_for_players([1]);
        let report = runtime
            .run_tick([command.clone(), command])
            .expect("tick should commit");

        assert_eq!(report.results.len(), 1);
        assert_eq!(report.results[0].disposition, CommandDisposition::Committed);
        assert_eq!(report.events.len(), 1);
        assert_eq!(report.snapshot.command_ledgers.len(), 1);
        assert_eq!(
            report.stage_trace[0],
            StageTraceEntry {
                stage: TransactionStage::IngressAdmission,
                received: 2,
                accepted: 1,
                rejected: 0,
                committed: 0,
                deduplicated: 1,
            }
        );
    }

    #[test]
    fn committed_retry_is_deduplicated_without_second_event() {
        let command = command(1, 1, 0, 0);
        let mut runtime = runtime_for_players([1]);
        runtime
            .run_tick([command.clone()])
            .expect("first tick should commit");
        let report = runtime
            .run_tick([command])
            .expect("retry tick should complete");

        assert_eq!(
            report.results[0].disposition,
            CommandDisposition::Deduplicated
        );
        assert!(report.events.is_empty());
        assert_eq!(report.snapshot.committed_event_count, 1);
        assert_eq!(report.snapshot.command_ledgers[0].last_sequence, 0);
    }

    #[test]
    fn conflicting_equivalence_class_is_rejected_before_mutation() {
        let first = command(1, 1, 0, 0);
        let second = command(1, 1, 0, 1);
        let mut runtime = runtime_for_players([1]);
        let report = runtime
            .run_tick([first, second])
            .expect("collision is a stable rejection");

        assert_eq!(report.results.len(), 2);
        assert!(report.results.iter().all(|result| {
            result.disposition
                == CommandDisposition::Rejected(RejectionCode::CommandSequenceCollision)
        }));
        assert!(report.events.is_empty());
        assert!(report.snapshot.command_ledgers.is_empty());
    }

    #[test]
    fn invalid_hash_variant_still_blocks_its_entire_equivalence_class() {
        let valid = command(1, 1, 0, 0);
        let mut conflicting = valid.clone();
        conflicting.command_id = CommandId::from_bytes([9; 16]);
        let mut runtime = runtime_for_players([1]);
        let report = runtime
            .run_tick([valid, conflicting])
            .expect("collision is a stable rejection");

        assert_eq!(report.results.len(), 2);
        assert!(report.results.iter().all(|result| {
            result.disposition
                == CommandDisposition::Rejected(RejectionCode::CommandSequenceCollision)
        }));
        assert!(report.events.is_empty());
        assert!(report.snapshot.command_ledgers.is_empty());
    }

    #[test]
    fn canonical_equivalent_capability_order_is_not_a_collision() {
        let mut first = command(1, 1, 0, 0);
        first.declared_capabilities = vec![
            noop_capability(),
            CapabilityId::new("world.read").expect("valid capability"),
        ];
        first
            .refresh_command_id()
            .expect("capability set is canonical");
        let mut second = first.clone();
        second.declared_capabilities.reverse();
        let mut runtime = RuntimeState::new(authority([player(1)]));
        let report = runtime
            .run_tick([first, second])
            .expect("denial is a stable result");

        assert_eq!(report.results.len(), 1);
        assert_eq!(
            report.results[0].disposition,
            CommandDisposition::Rejected(RejectionCode::CapabilityDenied)
        );
        assert!(report.snapshot.command_ledgers.is_empty());
    }

    #[test]
    fn sequence_reuse_does_not_replace_committed_ledger() {
        let mut runtime = runtime_for_players([1]);
        runtime
            .run_tick([command(1, 1, 5, 0)])
            .expect("sequence gaps are allowed");
        let report = runtime
            .run_tick([command(1, 1, 4, 1)])
            .expect("reuse is a stable rejection");

        assert_eq!(
            report.results[0].disposition,
            CommandDisposition::Rejected(RejectionCode::CommandSequenceReuse)
        );
        assert_eq!(report.snapshot.command_ledgers[0].last_sequence, 5);
        assert_eq!(report.snapshot.committed_event_count, 1);
    }

    #[test]
    fn exhausted_stream_rejects_any_nonduplicate_command() {
        let mut runtime = runtime_for_players([1]);
        runtime
            .run_tick([command(1, 1, u64::MAX, 0)])
            .expect("maximum sequence is representable");
        let report = runtime
            .run_tick([command(1, 1, 0, 1)])
            .expect("exhaustion is a stable rejection");

        assert_eq!(
            report.results[0].disposition,
            CommandDisposition::Rejected(RejectionCode::CommandSequenceExhausted)
        );
        assert_eq!(report.snapshot.command_ledgers[0].last_sequence, u64::MAX);
    }

    #[test]
    fn mismatched_command_id_is_rejected_without_ledger_mutation() {
        let mut command = command(1, 1, 0, 0);
        command.command_id = CommandId::from_bytes([9; 16]);
        let mut runtime = runtime_for_players([1]);
        let report = runtime
            .run_tick([command])
            .expect("invalid command is a stable rejection");

        assert_eq!(
            report.results[0].disposition,
            CommandDisposition::Rejected(RejectionCode::CommandIdMismatch)
        );
        assert!(report.events.is_empty());
        assert!(report.snapshot.command_ledgers.is_empty());
    }

    #[test]
    fn public_ingress_cannot_inject_outcome_phase() {
        let mut command = command(1, 1, 0, 0);
        command.phase = CommandPhase::Outcome;
        command
            .refresh_command_id()
            .expect("outcome command bytes are canonical");
        let mut runtime = runtime_for_players([1]);
        let report = runtime
            .run_tick([command])
            .expect("forbidden phase is a stable rejection");

        assert_eq!(
            report.results[0].disposition,
            CommandDisposition::Rejected(RejectionCode::PublicOutcomeForbidden)
        );
        assert!(report.snapshot.command_ledgers.is_empty());
    }

    #[test]
    fn unregistered_principal_is_rejected_before_mutation() {
        let mut runtime = RuntimeState::default();
        let report = runtime
            .run_tick([command(1, 1, 0, 0)])
            .expect("authentication failure is a stable rejection");

        assert_eq!(
            report.results[0].disposition,
            CommandDisposition::Rejected(RejectionCode::IssuerUnauthenticated)
        );
        assert_eq!(
            RejectionCode::IssuerUnauthenticated.as_str(),
            "ISSUER_UNAUTHENTICATED"
        );
    }

    #[test]
    fn missing_required_capability_is_rejected_before_mutation() {
        let mut command = command(1, 1, 0, 0);
        command.declared_capabilities.clear();
        command
            .refresh_command_id()
            .expect("command remains canonical");
        let mut runtime = runtime_for_players([1]);
        let report = runtime
            .run_tick([command])
            .expect("missing capability is a stable rejection");

        assert_eq!(
            report.results[0].disposition,
            CommandDisposition::Rejected(RejectionCode::CapabilityRequired)
        );
        assert!(report.events.is_empty());
    }

    #[test]
    fn granted_capability_is_required_not_only_declared() {
        let mut empty_authority = AuthorityRegistry::new();
        empty_authority
            .register(player(1), [])
            .expect("test principal is unique");
        let mut runtime = RuntimeState::new(empty_authority);
        let report = runtime
            .run_tick([command(1, 1, 0, 0)])
            .expect("denied capability is a stable rejection");

        assert_eq!(
            report.results[0].disposition,
            CommandDisposition::Rejected(RejectionCode::CapabilityDenied)
        );
    }

    #[test]
    fn unknown_schema_alias_cannot_spoof_registry_priority() {
        let mut command = command(1, 1, 0, 0);
        command.schema_id =
            SchemaId::new("nextengine.command.noop.high-priority").expect("valid schema ID");
        command
            .refresh_command_id()
            .expect("unknown schema command is still canonical");
        let mut runtime = runtime_for_players([1]);
        let report = runtime
            .run_tick([command])
            .expect("unknown schema is a stable rejection");

        assert_eq!(
            report.results[0].disposition,
            CommandDisposition::Rejected(RejectionCode::SchemaMismatch)
        );
    }

    #[test]
    fn precondition_uses_phase_start_authoritative_revision() {
        let mut matching = command(1, 1, 0, 0);
        matching.precondition_revision = Some(0);
        matching
            .refresh_command_id()
            .expect("matching precondition is canonical");
        let mut runtime = runtime_for_players([1]);
        let report = runtime
            .run_tick([matching])
            .expect("matching precondition should commit");
        assert_eq!(report.snapshot.authoritative_revision, 1);

        let mut stale = command(1, 1, 1, 1);
        stale.precondition_revision = Some(0);
        stale
            .refresh_command_id()
            .expect("stale precondition is canonical");
        let report = runtime
            .run_tick([stale])
            .expect("stale precondition is a stable rejection");
        assert_eq!(
            report.results[0].disposition,
            CommandDisposition::Rejected(RejectionCode::PreconditionFailed)
        );
        assert_eq!(report.snapshot.authoritative_revision, 1);
    }

    struct OneOutcome {
        system_id: SystemId,
        stream_id: CommandStreamId,
        sequence: u64,
    }

    impl OutcomeProvider for OneOutcome {
        fn collect(
            &mut self,
            context: OutcomeContext<'_>,
            sink: &mut OutcomeSink,
        ) -> Result<(), OutcomeCollectionError> {
            assert_eq!(context.ingress_events.len(), 1);
            sink.submit(
                OutcomeProposal::noop(self.system_id.clone(), self.stream_id, self.sequence)
                    .with_precondition_revision(context.authoritative_revision),
            );
            Ok(())
        }
    }

    #[test]
    fn authenticated_internal_outcome_commits_in_closed_second_phase() {
        let system_id = SystemId::new("runtime.physics-outcome").expect("valid system ID");
        let system_principal = IssuerPrincipal::InternalSystem(system_id.clone());
        let mut runtime = RuntimeState::new(authority([player(1), system_principal]));
        let mut provider = OneOutcome {
            system_id,
            stream_id: CommandStreamId::from_bytes([8; 16]),
            sequence: 0,
        };

        let report = runtime
            .run_tick_with_outcomes([command(1, 1, 0, 0)], &mut provider)
            .expect("both phases should commit");

        assert_eq!(report.events.len(), 2);
        assert_eq!(report.snapshot.authoritative_revision, 2);
        assert_eq!(
            report
                .stage_trace
                .iter()
                .map(|entry| entry.stage)
                .collect::<Vec<_>>(),
            vec![
                TransactionStage::IngressAdmission,
                TransactionStage::IngressCommit,
                TransactionStage::OutcomeCollection,
                TransactionStage::OutcomeAdmission,
                TransactionStage::OutcomeCommit,
                TransactionStage::SnapshotPublication,
            ]
        );
        assert_eq!(report.stage_trace[4].committed, 1);
    }

    struct ReentrantOutcome;

    impl OutcomeProvider for ReentrantOutcome {
        fn collect(
            &mut self,
            _context: OutcomeContext<'_>,
            sink: &mut OutcomeSink,
        ) -> Result<(), OutcomeCollectionError> {
            sink.enter_same_tick_batch()
        }
    }

    #[test]
    fn outcome_reentry_is_fatal_and_rolls_back_staged_ingress() {
        let mut runtime = runtime_for_players([1]);
        let before = runtime.snapshot();
        let error = runtime
            .run_tick_with_outcomes([command(1, 1, 0, 0)], &mut ReentrantOutcome)
            .expect_err("same-tick outcome re-entry must fail");

        assert_eq!(
            error,
            RuntimeFatalError::OutcomeCollection(OutcomeCollectionError::OutcomeReentryForbidden)
        );
        assert_eq!(error.stable_code(), "OUTCOME_REENTRY_FORBIDDEN");
        assert_eq!(runtime.snapshot(), before);
    }

    #[test]
    fn fatal_commit_error_rolls_back_all_staged_authoritative_state() {
        let mut runtime = runtime_for_players([1]);
        runtime.committed_event_count = u64::MAX;
        let before = runtime.snapshot();
        let error = runtime
            .run_tick([command(1, 1, 0, 0)])
            .expect_err("event exhaustion must be fatal");

        assert_eq!(error, RuntimeFatalError::EventCountExhausted);
        assert_eq!(runtime.snapshot(), before);
    }

    #[test]
    fn wrong_target_tick_is_rejected_without_ledger_mutation() {
        let mut runtime = runtime_for_players([1]);
        let report = runtime
            .run_tick([command(1, 1, 0, 3)])
            .expect("wrong tick is a stable rejection");

        assert_eq!(
            report.results[0].disposition,
            CommandDisposition::Rejected(RejectionCode::TargetTickMismatch)
        );
        assert!(report.snapshot.command_ledgers.is_empty());
    }
}
