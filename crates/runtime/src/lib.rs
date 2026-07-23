#![forbid(unsafe_code)]

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::{
    COMMAND_SCHEMA_VERSION, CanonicalError, CommandId, CommandLedgerSnapshot, CommandPayload,
    CommandPhase, CommandStreamId, DomainEvent, IssuerPrincipal, NOOP_COMMAND_SCHEMA_ID,
    RuntimeSnapshot, WorldCommand,
};

type LedgerKey = (CommandStreamId, IssuerPrincipal);
type CollisionKey = (CommandStreamId, IssuerPrincipal, u64);

#[derive(Clone, Debug, Default)]
pub struct RuntimeState {
    next_tick: u64,
    committed_event_count: u64,
    ledgers: BTreeMap<LedgerKey, LedgerEntry>,
}

impl RuntimeState {
    #[must_use]
    pub const fn next_tick(&self) -> u64 {
        self.next_tick
    }

    #[must_use]
    pub fn snapshot(&self) -> RuntimeSnapshot {
        snapshot_from(self.next_tick, self.committed_event_count, &self.ledgers)
    }

    pub fn run_tick(
        &mut self,
        commands: impl IntoIterator<Item = WorldCommand>,
    ) -> Result<TickReport, RuntimeFatalError> {
        let following_tick = self
            .next_tick
            .checked_add(1)
            .ok_or(RuntimeFatalError::TickExhausted)?;
        let tick = self.next_tick;
        let mut commands: Vec<_> = commands.into_iter().collect();
        commands.sort();
        commands.dedup();

        let mut results = Vec::new();
        let mut collision_groups: BTreeMap<CollisionKey, Vec<WorldCommand>> = BTreeMap::new();
        for command in commands {
            collision_groups
                .entry((command.stream_id, command.issuer.clone(), command.sequence))
                .or_default()
                .push(command);
        }

        let mut candidates = Vec::new();
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
                results.extend(group.into_iter().map(|command| {
                    let order_key = CommandOrderKey::from_command(&command);
                    OrderedResult::rejected(
                        order_key,
                        command.command_id,
                        command.sequence,
                        RejectionCode::CommandSequenceCollision,
                    )
                }));
            } else if let Some(command) = group.into_iter().next() {
                let order_key = CommandOrderKey::from_command(&command);
                let command_id = command.command_id;
                let sequence = command.sequence;
                let canonical_bytes = canonical_group
                    .into_iter()
                    .next()
                    .expect("nonempty collision group has canonical result");
                match validate_command(command, canonical_bytes) {
                    Ok(candidate) => candidates.push(candidate),
                    Err(code) => results.push(OrderedResult::rejected(
                        order_key, command_id, sequence, code,
                    )),
                }
            }
        }
        candidates.sort_by(compare_validated_commands);

        let mut staged_ledgers = self.ledgers.clone();
        let mut staged_event_count = self.committed_event_count;
        let mut events = Vec::new();
        for candidate in candidates {
            let command = candidate.command;
            let ledger_key = (command.stream_id, command.issuer.clone());
            if let Some(entry) = staged_ledgers.get(&ledger_key) {
                if command.sequence == entry.last_sequence
                    && command.command_id == entry.command_id
                    && candidate.canonical_bytes == entry.canonical_bytes
                {
                    results.push(OrderedResult::deduplicated(
                        candidate.order_key,
                        command.command_id,
                        command.sequence,
                    ));
                    continue;
                }
                if entry.last_sequence == u64::MAX {
                    results.push(OrderedResult::rejected(
                        candidate.order_key,
                        command.command_id,
                        command.sequence,
                        RejectionCode::CommandSequenceExhausted,
                    ));
                    continue;
                }
                if command.sequence <= entry.last_sequence {
                    results.push(OrderedResult::rejected(
                        candidate.order_key,
                        command.command_id,
                        command.sequence,
                        RejectionCode::CommandSequenceReuse,
                    ));
                    continue;
                }
            }
            if command.target_tick != tick {
                results.push(OrderedResult::rejected(
                    candidate.order_key,
                    command.command_id,
                    command.sequence,
                    RejectionCode::TargetTickMismatch,
                ));
                continue;
            }

            let event = DomainEvent::command_committed(tick, command.command_id, command.sequence)
                .map_err(RuntimeFatalError::InternalCanonicalization)?;
            staged_event_count = staged_event_count
                .checked_add(1)
                .ok_or(RuntimeFatalError::EventCountExhausted)?;
            staged_ledgers.insert(
                ledger_key,
                LedgerEntry {
                    last_sequence: command.sequence,
                    command_id: command.command_id,
                    canonical_bytes: candidate.canonical_bytes,
                },
            );
            results.push(OrderedResult::committed(
                candidate.order_key,
                command.command_id,
                command.sequence,
            ));
            events.push(event);
        }

        results.sort();
        let results = results.into_iter().map(|result| result.result).collect();
        let snapshot = snapshot_from(following_tick, staged_event_count, &staged_ledgers);
        self.next_tick = following_tick;
        self.committed_event_count = staged_event_count;
        self.ledgers = staged_ledgers;

        Ok(TickReport {
            tick,
            results,
            events,
            snapshot,
        })
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

fn validate_command(
    command: WorldCommand,
    canonical_bytes: Result<Vec<u8>, CanonicalError>,
) -> Result<ValidatedCommand, RejectionCode> {
    if command.schema_id.as_str() != NOOP_COMMAND_SCHEMA_ID
        || command.schema_version != COMMAND_SCHEMA_VERSION
        || command.payload != CommandPayload::Noop
    {
        return Err(RejectionCode::SchemaMismatch);
    }
    if command.phase != CommandPhase::Ingress {
        return Err(RejectionCode::CommandPhaseForbidden);
    }
    if !command.declared_capabilities.is_empty() {
        return Err(RejectionCode::CapabilityDenied);
    }
    if command.precondition_revision.is_some() {
        return Err(RejectionCode::PreconditionFailed);
    }
    if command.target.is_some() {
        return Err(RejectionCode::TargetNotAllowed);
    }
    let canonical_bytes = match canonical_bytes {
        Ok(bytes) => bytes,
        Err(_) => return Err(RejectionCode::CanonicalCommandInvalid),
    };
    let expected_id = match command.compute_command_id() {
        Ok(command_id) => command_id,
        Err(_) => return Err(RejectionCode::CanonicalCommandInvalid),
    };
    if command.command_id != expected_id {
        return Err(RejectionCode::CommandIdMismatch);
    }
    let order_key = CommandOrderKey::from_command(&command);
    Ok(ValidatedCommand {
        command,
        canonical_bytes,
        order_key,
    })
}

fn compare_validated_commands(left: &ValidatedCommand, right: &ValidatedCommand) -> Ordering {
    left.order_key.cmp(&right.order_key)
}

fn snapshot_from(
    next_tick: u64,
    committed_event_count: u64,
    ledgers: &BTreeMap<LedgerKey, LedgerEntry>,
) -> RuntimeSnapshot {
    RuntimeSnapshot {
        next_tick,
        committed_event_count,
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
    fn from_command(command: &WorldCommand) -> Self {
        Self {
            target_tick: command.target_tick,
            phase: command.phase,
            priority_class: priority_class(&command.payload),
            issuer_tag: command.issuer.tag(),
            issuer_id_bytes: command.issuer.identifier_bytes().to_vec(),
            sequence: command.sequence,
            command_id: command.command_id,
        }
    }
}

const fn priority_class(payload: &CommandPayload) -> u16 {
    match payload {
        CommandPayload::Noop => 0,
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RejectionCode {
    SchemaMismatch,
    CanonicalCommandInvalid,
    CommandIdMismatch,
    CommandPhaseForbidden,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TickReport {
    pub tick: u64,
    pub results: Vec<CommandResult>,
    pub events: Vec<DomainEvent>,
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
    InternalCanonicalization(CanonicalError),
}

impl Display for RuntimeFatalError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TickExhausted => formatter.write_str("simulation tick counter exhausted"),
            Self::EventCountExhausted => formatter.write_str("committed event counter exhausted"),
            Self::InternalCanonicalization(error) => {
                write!(formatter, "internal canonicalization failed: {error}")
            }
        }
    }
}

impl Error for RuntimeFatalError {}

#[cfg(test)]
mod tests {
    use next_contracts::{
        CommandId, CommandPhase, CommandStreamId, IssuerPrincipal, PlayerPrincipalId, WorldCommand,
    };

    use super::{CommandDisposition, RejectionCode, RuntimeState};

    fn command(stream: u8, issuer: u8, sequence: u64, target_tick: u64) -> WorldCommand {
        WorldCommand::noop(
            CommandStreamId::from_bytes([stream; 16]),
            IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([issuer; 16])),
            sequence,
            target_tick,
        )
        .expect("test command is canonical")
    }

    #[test]
    fn arrival_permutations_produce_identical_reports() {
        let first = command(1, 2, 0, 0);
        let second = command(2, 1, 0, 0);
        let mut left = RuntimeState::default();
        let mut right = RuntimeState::default();

        let left_report = left
            .run_tick([first.clone(), second.clone()])
            .expect("tick should commit");
        let right_report = right.run_tick([second, first]).expect("tick should commit");

        assert_eq!(left_report, right_report);
        assert_eq!(left.snapshot(), right.snapshot());
    }

    #[test]
    fn exact_batch_duplicates_commit_once() {
        let command = command(1, 1, 0, 0);
        let mut runtime = RuntimeState::default();
        let report = runtime
            .run_tick([command.clone(), command])
            .expect("tick should commit");

        assert_eq!(report.results.len(), 1);
        assert_eq!(report.results[0].disposition, CommandDisposition::Committed);
        assert_eq!(report.events.len(), 1);
        assert_eq!(report.snapshot.command_ledgers.len(), 1);
    }

    #[test]
    fn committed_retry_is_deduplicated_without_second_event() {
        let command = command(1, 1, 0, 0);
        let mut runtime = RuntimeState::default();
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
        let mut runtime = RuntimeState::default();
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
        let mut runtime = RuntimeState::default();
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
    fn canonical_equivalent_metadata_order_is_not_a_collision() {
        let mut first = command(1, 1, 0, 0);
        first.declared_capabilities = vec![
            next_contracts::CapabilityId::new("world.read").expect("valid capability"),
            next_contracts::CapabilityId::new("world.write").expect("valid capability"),
        ];
        first
            .refresh_command_id()
            .expect("capability set is canonical");
        let mut second = first.clone();
        second.declared_capabilities.reverse();
        let mut runtime = RuntimeState::default();
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
        let mut runtime = RuntimeState::default();
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
        let mut runtime = RuntimeState::default();
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
        let mut runtime = RuntimeState::default();
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
        let mut runtime = RuntimeState::default();
        let report = runtime
            .run_tick([command])
            .expect("forbidden phase is a stable rejection");

        assert_eq!(
            report.results[0].disposition,
            CommandDisposition::Rejected(RejectionCode::CommandPhaseForbidden)
        );
        assert!(report.snapshot.command_ledgers.is_empty());
    }

    #[test]
    fn wrong_target_tick_is_rejected_without_ledger_mutation() {
        let mut runtime = RuntimeState::default();
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
