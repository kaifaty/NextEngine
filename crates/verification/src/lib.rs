#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::{
    CanonicalError, DomainEvent, RuntimeSnapshot, SchemaId, StateRoot, WorldCommand, sha256,
};
use next_runtime::{CommandResult, RuntimeFatalError, RuntimeState};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateSegment {
    pub owner_id: SchemaId,
    pub schema_id: SchemaId,
    pub segment_id: SchemaId,
    pub canonical_bytes: Vec<u8>,
}

impl StateSegment {
    #[must_use]
    pub fn new(
        owner_id: SchemaId,
        schema_id: SchemaId,
        segment_id: SchemaId,
        canonical_bytes: Vec<u8>,
    ) -> Self {
        Self {
            owner_id,
            schema_id,
            segment_id,
            canonical_bytes,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StateRootError {
    DuplicateSegment,
    LengthOverflow,
}

impl Display for StateRootError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::DuplicateSegment => "duplicate owner/schema/segment tuple in authoritative state",
            Self::LengthOverflow => "state segment identifier or payload exceeds canonical length",
        })
    }
}

impl Error for StateRootError {}

pub fn compute_state_root(
    segments: impl IntoIterator<Item = StateSegment>,
) -> Result<StateRoot, StateRootError> {
    let mut segments: Vec<_> = segments.into_iter().collect();
    segments.sort_by(|left, right| {
        (
            left.owner_id.as_str(),
            left.schema_id.as_str(),
            left.segment_id.as_str(),
        )
            .cmp(&(
                right.owner_id.as_str(),
                right.schema_id.as_str(),
                right.segment_id.as_str(),
            ))
    });
    if segments.windows(2).any(|pair| {
        pair[0].owner_id == pair[1].owner_id
            && pair[0].schema_id == pair[1].schema_id
            && pair[0].segment_id == pair[1].segment_id
    }) {
        return Err(StateRootError::DuplicateSegment);
    }

    let leaf_count = u64::try_from(segments.len()).map_err(|_| StateRootError::LengthOverflow)?;
    let mut nodes = Vec::with_capacity(segments.len());
    for segment in segments {
        let mut segment_preimage = Vec::new();
        segment_preimage.extend_from_slice(b"nextengine.state-segment.v1\0");
        segment_preimage.extend_from_slice(
            &u64::try_from(segment.canonical_bytes.len())
                .map_err(|_| StateRootError::LengthOverflow)?
                .to_le_bytes(),
        );
        segment_preimage.extend_from_slice(&segment.canonical_bytes);
        let segment_hash = sha256(&segment_preimage);

        let mut leaf_preimage = Vec::new();
        leaf_preimage.extend_from_slice(b"nextengine.state-leaf.v1\0");
        extend_identifier(&mut leaf_preimage, segment.owner_id.as_str())?;
        extend_identifier(&mut leaf_preimage, segment.schema_id.as_str())?;
        extend_identifier(&mut leaf_preimage, segment.segment_id.as_str())?;
        leaf_preimage.extend_from_slice(&segment_hash);
        nodes.push(sha256(&leaf_preimage));
    }

    let merkle_root = if nodes.is_empty() {
        sha256(b"nextengine.state-empty.v1\0")
    } else {
        while nodes.len() > 1 {
            let mut parents = Vec::with_capacity(nodes.len().div_ceil(2));
            for pair in nodes.chunks(2) {
                let mut preimage = Vec::new();
                if let [left, right] = pair {
                    preimage.extend_from_slice(b"nextengine.state-node.v1\0");
                    preimage.extend_from_slice(left);
                    preimage.extend_from_slice(right);
                } else {
                    preimage.extend_from_slice(b"nextengine.state-carry.v1\0");
                    preimage.extend_from_slice(&pair[0]);
                }
                parents.push(sha256(&preimage));
            }
            nodes = parents;
        }
        nodes[0]
    };

    let mut root_preimage = Vec::new();
    root_preimage.extend_from_slice(b"nextengine.state-root.v1\0");
    root_preimage.extend_from_slice(&leaf_count.to_le_bytes());
    root_preimage.extend_from_slice(&merkle_root);
    Ok(StateRoot::from_bytes(sha256(&root_preimage)))
}

fn extend_identifier(target: &mut Vec<u8>, value: &str) -> Result<(), StateRootError> {
    target.extend_from_slice(
        &u32::try_from(value.len())
            .map_err(|_| StateRootError::LengthOverflow)?
            .to_le_bytes(),
    );
    target.extend_from_slice(value.as_bytes());
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayInput {
    pub ticks: Vec<ReplayTickInput>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayTickInput {
    pub commands: Vec<WorldCommand>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayOutput {
    pub ticks: Vec<ReplayTickRecord>,
    pub final_snapshot: RuntimeSnapshot,
}

impl ReplayOutput {
    #[must_use]
    pub fn final_state_root(&self) -> Option<StateRoot> {
        self.ticks.last().map(|tick| tick.state_root)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayTickRecord {
    pub tick: u64,
    pub command_results: Vec<CommandResult>,
    pub events: Vec<DomainEvent>,
    pub state_root: StateRoot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplayError {
    Runtime(RuntimeFatalError),
    SnapshotCanonicalization(CanonicalError),
    StateRoot(StateRootError),
    NondeterministicResult {
        first_divergent_tick: u64,
        expected_state_root: Option<StateRoot>,
        actual_state_root: Option<StateRoot>,
    },
}

impl ReplayError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::Runtime(_) => "REPLAY_RUNTIME_FATAL",
            Self::SnapshotCanonicalization(_) => "REPLAY_SNAPSHOT_CANONICALIZATION_FAILED",
            Self::StateRoot(_) => "REPLAY_STATE_ROOT_FAILED",
            Self::NondeterministicResult { .. } => "NONDETERMINISTIC_RESULT",
        }
    }
}

impl Display for ReplayError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Runtime(error) => write!(formatter, "runtime failed during replay: {error}"),
            Self::SnapshotCanonicalization(error) => {
                write!(formatter, "snapshot canonicalization failed: {error}")
            }
            Self::StateRoot(error) => write!(formatter, "state-root computation failed: {error}"),
            Self::NondeterministicResult {
                first_divergent_tick,
                expected_state_root,
                actual_state_root,
            } => write!(
                formatter,
                "NONDETERMINISTIC_RESULT at tick {first_divergent_tick}: expected {}, actual {}",
                optional_root_hex(*expected_state_root),
                optional_root_hex(*actual_state_root)
            ),
        }
    }
}

impl Error for ReplayError {}

impl From<RuntimeFatalError> for ReplayError {
    fn from(error: RuntimeFatalError) -> Self {
        Self::Runtime(error)
    }
}

impl From<CanonicalError> for ReplayError {
    fn from(error: CanonicalError) -> Self {
        Self::SnapshotCanonicalization(error)
    }
}

impl From<StateRootError> for ReplayError {
    fn from(error: StateRootError) -> Self {
        Self::StateRoot(error)
    }
}

pub fn run_replay(input: &ReplayInput) -> Result<ReplayOutput, ReplayError> {
    let mut runtime = RuntimeState::default();
    let mut records = Vec::with_capacity(input.ticks.len());
    for tick in &input.ticks {
        let report = runtime.run_tick(tick.commands.clone())?;
        let canonical_snapshot = report.snapshot.canonical_bytes()?;
        let state_root = compute_state_root([StateSegment::new(
            SchemaId::new("runtime").map_err(CanonicalError::InvalidIdentifier)?,
            SchemaId::new("nextengine.runtime.snapshot")
                .map_err(CanonicalError::InvalidIdentifier)?,
            SchemaId::new("command-ledger").map_err(CanonicalError::InvalidIdentifier)?,
            canonical_snapshot,
        )])?;
        records.push(ReplayTickRecord {
            tick: report.tick,
            command_results: report.results,
            events: report.events,
            state_root,
        });
    }
    Ok(ReplayOutput {
        ticks: records,
        final_snapshot: runtime.snapshot(),
    })
}

pub fn verify_replay(expected: &ReplayOutput, input: &ReplayInput) -> Result<(), ReplayError> {
    let actual = run_replay(input)?;
    compare_replay_outputs(expected, &actual)
}

pub fn compare_replay_outputs(
    expected: &ReplayOutput,
    actual: &ReplayOutput,
) -> Result<(), ReplayError> {
    let shared_ticks = expected.ticks.len().min(actual.ticks.len());
    for index in 0..shared_ticks {
        if expected.ticks[index] != actual.ticks[index] {
            return Err(ReplayError::NondeterministicResult {
                first_divergent_tick: expected.ticks[index].tick.min(actual.ticks[index].tick),
                expected_state_root: Some(expected.ticks[index].state_root),
                actual_state_root: Some(actual.ticks[index].state_root),
            });
        }
    }
    if expected.ticks.len() != actual.ticks.len() {
        let index = shared_ticks;
        return Err(ReplayError::NondeterministicResult {
            first_divergent_tick: u64::try_from(index).unwrap_or(u64::MAX),
            expected_state_root: expected.ticks.get(index).map(|tick| tick.state_root),
            actual_state_root: actual.ticks.get(index).map(|tick| tick.state_root),
        });
    }
    if expected.final_snapshot != actual.final_snapshot {
        let first_divergent_tick = expected
            .ticks
            .last()
            .map_or(0, |tick| tick.tick.saturating_add(1));
        return Err(ReplayError::NondeterministicResult {
            first_divergent_tick,
            expected_state_root: expected.final_state_root(),
            actual_state_root: actual.final_state_root(),
        });
    }
    Ok(())
}

fn optional_root_hex(root: Option<StateRoot>) -> String {
    root.map_or_else(|| "absent".to_owned(), StateRoot::to_hex)
}

#[cfg(test)]
mod tests {
    use next_contracts::{
        CommandStreamId, IssuerPrincipal, PlayerPrincipalId, SchemaId, WorldCommand,
    };

    use super::{
        ReplayError, ReplayInput, ReplayTickInput, StateSegment, compare_replay_outputs,
        compute_state_root, run_replay, verify_replay,
    };

    fn command(stream: u8, issuer: u8, sequence: u64, tick: u64) -> WorldCommand {
        WorldCommand::noop(
            CommandStreamId::from_bytes([stream; 16]),
            IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([issuer; 16])),
            sequence,
            tick,
        )
        .expect("test command is canonical")
    }

    fn scenario() -> ReplayInput {
        ReplayInput {
            ticks: vec![
                ReplayTickInput {
                    commands: vec![command(1, 2, 0, 0), command(2, 1, 0, 0)],
                },
                ReplayTickInput {
                    commands: vec![command(1, 2, 1, 1)],
                },
            ],
        }
    }

    #[test]
    fn empty_state_root_matches_golden_vector() {
        assert_eq!(
            compute_state_root([])
                .expect("empty state tree is valid")
                .to_hex(),
            "a03902d5430adb91603bcd477a7b814b5c08ecde092f04f8c4ff359032ffca73"
        );
    }

    #[test]
    fn state_segment_order_does_not_change_root() {
        let first = StateSegment::new(
            SchemaId::new("a").expect("valid owner"),
            SchemaId::new("schema").expect("valid schema"),
            SchemaId::new("one").expect("valid segment"),
            vec![1],
        );
        let second = StateSegment::new(
            SchemaId::new("b").expect("valid owner"),
            SchemaId::new("schema").expect("valid schema"),
            SchemaId::new("two").expect("valid segment"),
            vec![2],
        );
        assert_eq!(
            compute_state_root([first.clone(), second.clone()]).expect("valid state"),
            compute_state_root([second, first]).expect("valid state")
        );
    }

    #[test]
    fn duplicate_state_segment_is_rejected() {
        let segment = StateSegment::new(
            SchemaId::new("runtime").expect("valid owner"),
            SchemaId::new("nextengine.runtime.snapshot").expect("valid schema"),
            SchemaId::new("command-ledger").expect("valid segment"),
            vec![1],
        );
        assert_eq!(
            compute_state_root([segment.clone(), segment]),
            Err(super::StateRootError::DuplicateSegment)
        );
    }

    #[test]
    fn replay_of_same_input_is_exact() {
        let input = scenario();
        let expected = run_replay(&input).expect("reference replay runs");
        verify_replay(&expected, &input).expect("same input must replay exactly");
    }

    #[test]
    fn arrival_permutation_preserves_replay_output() {
        let first = scenario();
        let mut second = first.clone();
        second.ticks[0].commands.reverse();
        let first_output = run_replay(&first).expect("first replay runs");
        let second_output = run_replay(&second).expect("second replay runs");
        compare_replay_outputs(&first_output, &second_output)
            .expect("arrival order must not affect replay");
    }

    #[test]
    fn replay_reports_first_divergent_tick() {
        let first = scenario();
        let mut second = first.clone();
        second.ticks[1].commands = vec![command(1, 2, 2, 1)];
        let first_output = run_replay(&first).expect("first replay runs");
        let second_output = run_replay(&second).expect("second replay runs");
        let error = compare_replay_outputs(&first_output, &second_output)
            .expect_err("different causal command must diverge");

        assert!(matches!(
            error,
            ReplayError::NondeterministicResult {
                first_divergent_tick: 1,
                ..
            }
        ));
        assert_eq!(error.stable_code(), "NONDETERMINISTIC_RESULT");
    }

    #[test]
    fn scenario_final_root_is_a_pinned_vector() {
        assert_eq!(
            run_replay(&scenario())
                .expect("scenario runs")
                .final_state_root()
                .expect("scenario has ticks")
                .to_hex(),
            "54525e4ed0a96b512d5eae0745c1722ab94ac04dcaa25ac6b6cf81e7f3db66b4"
        );
    }
}
