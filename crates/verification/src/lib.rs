#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::{
    CanonicalDecodeLimits, CanonicalError, CommandLedgerHash, DomainEvent, ManifestValidationError,
    RPG_SNAPSHOT_OWNER_ID, RPG_SNAPSHOT_SCHEMA_ID, RPG_SNAPSHOT_SEGMENT_ID,
    RUNTIME_SNAPSHOT_OWNER_ID, RUNTIME_SNAPSHOT_SCHEMA_ID, RUNTIME_SNAPSHOT_SEGMENT_ID,
    ReplayManifestV1, RpgSnapshot, RuntimeSnapshot, SchemaId, StateRoot, WorldCommand, sha256,
};
use next_runtime::{
    AuthorityRegistry, CommandResult, RuntimeFatalError, RuntimeState, SnapshotRestoreError,
};

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
    pub authority: AuthorityRegistry,
    pub ticks: Vec<ReplayTickInput>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RpgReplayInput {
    pub authority: AuthorityRegistry,
    pub initial_rpg_snapshot: RpgSnapshot,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RpgReplayOutput {
    pub ticks: Vec<ReplayTickRecord>,
    pub final_runtime_snapshot: RuntimeSnapshot,
    pub final_rpg_snapshot: RpgSnapshot,
}

impl RpgReplayOutput {
    #[must_use]
    pub fn final_state_root(&self) -> Option<StateRoot> {
        self.ticks.last().map(|tick| tick.state_root)
    }
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
    pub command_ledger_hash: CommandLedgerHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayComparePointMismatch {
    pub first_divergent_tick: u64,
    pub expected_state_root: StateRoot,
    pub actual_state_root: StateRoot,
    pub expected_command_ledger_hash: CommandLedgerHash,
    pub actual_command_ledger_hash: CommandLedgerHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplayError {
    Runtime(RuntimeFatalError),
    Manifest(ManifestValidationError),
    SnapshotRestore(SnapshotRestoreError),
    SnapshotCanonicalization(CanonicalError),
    StateRoot(StateRootError),
    InitialSnapshotMismatch {
        expected_state_root: StateRoot,
        actual_state_root: StateRoot,
    },
    ComparePointMismatch(Box<ReplayComparePointMismatch>),
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
            Self::Manifest(_) => "REPLAY_MANIFEST_INVALID",
            Self::SnapshotRestore(_) => "REPLAY_SNAPSHOT_RESTORE_FAILED",
            Self::SnapshotCanonicalization(_) => "REPLAY_SNAPSHOT_CANONICALIZATION_FAILED",
            Self::StateRoot(_) => "REPLAY_STATE_ROOT_FAILED",
            Self::InitialSnapshotMismatch { .. } => "REPLAY_INITIAL_SNAPSHOT_MISMATCH",
            Self::ComparePointMismatch { .. } => "NONDETERMINISTIC_RESULT",
            Self::NondeterministicResult { .. } => "NONDETERMINISTIC_RESULT",
        }
    }
}

impl Display for ReplayError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Runtime(error) => write!(formatter, "runtime failed during replay: {error}"),
            Self::Manifest(error) => write!(formatter, "replay manifest is invalid: {error}"),
            Self::SnapshotRestore(error) => {
                write!(formatter, "replay snapshot restore failed: {error}")
            }
            Self::SnapshotCanonicalization(error) => {
                write!(formatter, "snapshot canonicalization failed: {error}")
            }
            Self::StateRoot(error) => write!(formatter, "state-root computation failed: {error}"),
            Self::InitialSnapshotMismatch {
                expected_state_root,
                actual_state_root,
            } => write!(
                formatter,
                "REPLAY_INITIAL_SNAPSHOT_MISMATCH: expected {}, actual {}",
                expected_state_root.to_hex(),
                actual_state_root.to_hex()
            ),
            Self::ComparePointMismatch(mismatch) => write!(
                formatter,
                "NONDETERMINISTIC_RESULT at tick {}: state root {} != {}; command ledger {} != {}",
                mismatch.first_divergent_tick,
                mismatch.expected_state_root.to_hex(),
                mismatch.actual_state_root.to_hex(),
                mismatch.expected_command_ledger_hash.to_hex(),
                mismatch.actual_command_ledger_hash.to_hex()
            ),
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

impl From<ManifestValidationError> for ReplayError {
    fn from(error: ManifestValidationError) -> Self {
        Self::Manifest(error)
    }
}

impl From<SnapshotRestoreError> for ReplayError {
    fn from(error: SnapshotRestoreError) -> Self {
        Self::SnapshotRestore(error)
    }
}

impl From<StateRootError> for ReplayError {
    fn from(error: StateRootError) -> Self {
        Self::StateRoot(error)
    }
}

pub fn run_replay(input: &ReplayInput) -> Result<ReplayOutput, ReplayError> {
    let mut runtime = RuntimeState::new(input.authority.clone());
    let mut records = Vec::with_capacity(input.ticks.len());
    for tick in &input.ticks {
        let report = runtime.run_tick(tick.commands.clone())?;
        let state_root = compute_runtime_snapshot_root(&report.snapshot)?;
        let command_ledger_hash = report.snapshot.command_ledger_hash()?;
        records.push(ReplayTickRecord {
            tick: report.tick,
            command_results: report.results,
            events: report.events,
            state_root,
            command_ledger_hash,
        });
    }
    Ok(ReplayOutput {
        ticks: records,
        final_snapshot: runtime.snapshot(),
    })
}

pub fn run_rpg_replay(input: &RpgReplayInput) -> Result<RpgReplayOutput, ReplayError> {
    let mut runtime = RuntimeState::with_rpg_snapshot(
        input.authority.clone(),
        input.initial_rpg_snapshot.clone(),
    )?;
    let mut records = Vec::with_capacity(input.ticks.len());
    for tick in &input.ticks {
        let report = runtime.run_tick(tick.commands.clone())?;
        let state_root = compute_world_snapshot_root(&report.snapshot, &report.rpg_snapshot)?;
        let command_ledger_hash = report.snapshot.command_ledger_hash()?;
        records.push(ReplayTickRecord {
            tick: report.tick,
            command_results: report.results,
            events: report.events,
            state_root,
            command_ledger_hash,
        });
    }
    Ok(RpgReplayOutput {
        ticks: records,
        final_runtime_snapshot: runtime.snapshot(),
        final_rpg_snapshot: runtime.rpg_snapshot(),
    })
}

pub fn run_replay_manifest(manifest: &ReplayManifestV1) -> Result<ReplayOutput, ReplayError> {
    let limits = CanonicalDecodeLimits::default();
    let (initial_snapshot, decoded_ticks) = manifest.validate_and_decode(limits)?;

    let actual_initial_root = compute_runtime_snapshot_root(&initial_snapshot)?;
    if actual_initial_root != manifest.initial_state_root {
        return Err(ReplayError::InitialSnapshotMismatch {
            expected_state_root: manifest.initial_state_root,
            actual_state_root: actual_initial_root,
        });
    }

    let mut authority = AuthorityRegistry::new();
    for grant in &manifest.authority {
        authority
            .register(grant.principal.clone(), grant.capabilities.clone())
            .map_err(|_| ManifestValidationError::AuthorityNotStrictlySorted)?;
    }
    let mut runtime = RuntimeState::restore(initial_snapshot, authority)?;
    let mut records = Vec::with_capacity(decoded_ticks.len());
    for ((tick_manifest, commands), compare_point) in manifest
        .ticks
        .iter()
        .zip(decoded_ticks)
        .zip(&manifest.compare_points)
    {
        let report = runtime.run_tick(commands)?;
        let state_root = compute_runtime_snapshot_root(&report.snapshot)?;
        let command_ledger_hash = report.snapshot.command_ledger_hash()?;
        if state_root != compare_point.state_root
            || command_ledger_hash != compare_point.command_ledger_hash
        {
            return Err(ReplayError::ComparePointMismatch(Box::new(
                ReplayComparePointMismatch {
                    first_divergent_tick: tick_manifest.tick,
                    expected_state_root: compare_point.state_root,
                    actual_state_root: state_root,
                    expected_command_ledger_hash: compare_point.command_ledger_hash,
                    actual_command_ledger_hash: command_ledger_hash,
                },
            )));
        }
        records.push(ReplayTickRecord {
            tick: report.tick,
            command_results: report.results,
            events: report.events,
            state_root,
            command_ledger_hash,
        });
    }
    Ok(ReplayOutput {
        ticks: records,
        final_snapshot: runtime.snapshot(),
    })
}

pub fn compute_runtime_snapshot_root(snapshot: &RuntimeSnapshot) -> Result<StateRoot, ReplayError> {
    let canonical_snapshot = snapshot.canonical_bytes()?;
    Ok(compute_state_root([StateSegment::new(
        SchemaId::new(RUNTIME_SNAPSHOT_OWNER_ID).map_err(CanonicalError::InvalidIdentifier)?,
        SchemaId::new(RUNTIME_SNAPSHOT_SCHEMA_ID).map_err(CanonicalError::InvalidIdentifier)?,
        SchemaId::new(RUNTIME_SNAPSHOT_SEGMENT_ID).map_err(CanonicalError::InvalidIdentifier)?,
        canonical_snapshot,
    )])?)
}

pub fn compute_world_snapshot_root(
    runtime_snapshot: &RuntimeSnapshot,
    rpg_snapshot: &RpgSnapshot,
) -> Result<StateRoot, ReplayError> {
    Ok(compute_state_root([
        StateSegment::new(
            SchemaId::new(RUNTIME_SNAPSHOT_OWNER_ID).map_err(CanonicalError::InvalidIdentifier)?,
            SchemaId::new(RUNTIME_SNAPSHOT_SCHEMA_ID).map_err(CanonicalError::InvalidIdentifier)?,
            SchemaId::new(RUNTIME_SNAPSHOT_SEGMENT_ID)
                .map_err(CanonicalError::InvalidIdentifier)?,
            runtime_snapshot.canonical_bytes()?,
        ),
        StateSegment::new(
            SchemaId::new(RPG_SNAPSHOT_OWNER_ID).map_err(CanonicalError::InvalidIdentifier)?,
            SchemaId::new(RPG_SNAPSHOT_SCHEMA_ID).map_err(CanonicalError::InvalidIdentifier)?,
            SchemaId::new(RPG_SNAPSHOT_SEGMENT_ID).map_err(CanonicalError::InvalidIdentifier)?,
            rpg_snapshot.canonical_bytes()?,
        ),
    ])?)
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
        AuthorityGrant, CapabilityId, CommandStreamId, ContentHash, IssuerPrincipal,
        NOOP_COMMAND_CAPABILITY_ID, PlayerPrincipalId, REPLAY_MANIFEST_SCHEMA_VERSION,
        ReplayCommandRecord, ReplayComparePoint, ReplayManifestV1, ReplayTickManifest,
        RuntimeSnapshot, SaveCompatibility, SchemaId, StateRoot, TickSettings, WorldCommand,
    };
    use next_runtime::AuthorityRegistry;

    use super::{
        ReplayError, ReplayInput, ReplayTickInput, StateSegment, compare_replay_outputs,
        compute_runtime_snapshot_root, compute_state_root, run_replay, run_replay_manifest,
        verify_replay,
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
        let first = IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([2; 16]));
        let second = IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([1; 16]));
        let capability =
            CapabilityId::new(NOOP_COMMAND_CAPABILITY_ID).expect("built-in capability ID is valid");
        let mut authority = AuthorityRegistry::new();
        authority
            .register(first, [capability.clone()])
            .expect("first principal is unique");
        authority
            .register(second, [capability])
            .expect("second principal is unique");
        ReplayInput {
            authority,
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

    fn compatibility() -> SaveCompatibility {
        SaveCompatibility {
            engine_build_hash: ContentHash::from_bytes([1; 32]),
            game_build_hash: ContentHash::from_bytes([2; 32]),
            project_id: SchemaId::new("nextengine.replay-test").expect("valid project"),
            schema_registry_hash: ContentHash::from_bytes([3; 32]),
            content_manifest_hash: ContentHash::from_bytes([4; 32]),
            mechanics_lock_hash: ContentHash::from_bytes([5; 32]),
            tick_settings: TickSettings {
                gameplay_hz: 30,
                physics_hz: 120,
                motor_hz: 60,
            },
            loaded_chunk_revisions: vec![],
            rng_stream_states: vec![],
            physical_bindings: vec![],
            policy_state_schemas: vec![],
            plugin_script_bindings: vec![],
        }
    }

    fn replay_manifest() -> (ReplayManifestV1, super::ReplayOutput) {
        let input = scenario();
        let expected = run_replay(&input).expect("reference replay runs");
        let initial_snapshot = RuntimeSnapshot {
            next_tick: 0,
            committed_event_count: 0,
            authoritative_revision: 0,
            command_ledgers: vec![],
        };
        let authority = input
            .authority
            .entries()
            .map(|(principal, capabilities)| AuthorityGrant {
                principal: principal.clone(),
                capabilities: capabilities.iter().cloned().collect(),
            })
            .collect();
        let ticks = input
            .ticks
            .iter()
            .enumerate()
            .map(|(tick, input)| ReplayTickManifest {
                tick: u64::try_from(tick).expect("test tick fits u64"),
                commands: input
                    .commands
                    .iter()
                    .map(|command| {
                        ReplayCommandRecord::from_command(command)
                            .expect("test command is canonical")
                    })
                    .collect(),
            })
            .collect();
        let compare_points = expected
            .ticks
            .iter()
            .map(|tick| ReplayComparePoint {
                tick: tick.tick,
                state_root: tick.state_root,
                command_ledger_hash: tick.command_ledger_hash,
            })
            .collect();
        (
            ReplayManifestV1 {
                schema_version: REPLAY_MANIFEST_SCHEMA_VERSION,
                compatibility: compatibility(),
                initial_snapshot_bytes: initial_snapshot
                    .canonical_bytes()
                    .expect("initial snapshot is canonical"),
                initial_state_root: compute_runtime_snapshot_root(&initial_snapshot)
                    .expect("initial root computes"),
                authority,
                ticks,
                compare_points,
            },
            expected,
        )
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
    fn versioned_manifest_restores_snapshot_and_checks_every_compare_point() {
        let (manifest, expected) = replay_manifest();
        let actual = run_replay_manifest(&manifest).expect("manifest replay is exact");
        assert_eq!(actual, expected);
    }

    #[test]
    fn manifest_reports_first_ledger_or_state_divergence() {
        let (mut manifest, _) = replay_manifest();
        manifest.compare_points[0].state_root = StateRoot::from_bytes([9; 32]);
        let error = run_replay_manifest(&manifest).expect_err("compare point must fail");
        assert!(matches!(
            error,
            ReplayError::ComparePointMismatch(ref details)
                if details.first_divergent_tick == 0
        ));
        assert_eq!(error.stable_code(), "NONDETERMINISTIC_RESULT");
    }

    #[test]
    fn manifest_decodes_entire_command_stream_before_runtime_restore() {
        let (mut manifest, _) = replay_manifest();
        manifest.ticks[1].commands[0]
            .canonical_command_bytes
            .push(0);
        let error = run_replay_manifest(&manifest).expect_err("corrupt command must fail closed");
        assert!(matches!(error, ReplayError::Manifest(_)));
        assert_eq!(error.stable_code(), "REPLAY_MANIFEST_INVALID");
    }

    #[test]
    fn scenario_final_root_is_a_pinned_vector() {
        assert_eq!(
            run_replay(&scenario())
                .expect("scenario runs")
                .final_state_root()
                .expect("scenario has ticks")
                .to_hex(),
            "c029f729777971b98555cc9878ef6e9161fe3cfd8924c690d9cd16b7b64b0001"
        );
    }
}
