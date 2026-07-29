use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::{
    CanonicalDecodeLimits, CanonicalError, CommandLedgerHash, DomainEvent, ManifestValidationError,
    PHYSICS_SNAPSHOT_OWNER_ID, PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID,
    PHYSICS_WORLD_CHECKPOINT_SCHEMA_VERSION, PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID,
    RPG_AGGREGATE_SNAPSHOT_OWNER_ID, RPG_AGGREGATE_SNAPSHOT_SCHEMA_ID,
    RPG_AGGREGATE_SNAPSHOT_SEGMENT_ID, RUNTIME_SNAPSHOT_OWNER_ID, RUNTIME_SNAPSHOT_SCHEMA_ID,
    RUNTIME_SNAPSHOT_SEGMENT_ID, ReplayCommandResultV2, ReplayManifestV4, RpgSnapshotV2,
    RuntimeSnapshot, SaveSegmentDescriptor, SchemaId, StateRoot, WorldCheckpointV4, WorldCommand,
};
use next_runtime::{
    AuthorityRegistry, CommandResult, RuntimeBootstrapV3, RuntimeFatalError, RuntimeReplayDriver,
    RuntimeReplayError, RuntimeState, SnapshotRestoreError,
};

use crate::StateRootError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayInput {
    pub bootstrap: RuntimeBootstrapV3,
    pub authority: AuthorityRegistry,
    pub ticks: Vec<ReplayTickInput>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RpgReplayInput {
    pub bootstrap: RuntimeBootstrapV3,
    pub authority: AuthorityRegistry,
    pub initial_rpg_snapshot: RpgSnapshotV2,
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
    pub final_checkpoint: WorldCheckpointV4,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RpgReplayOutput {
    pub ticks: Vec<ReplayTickRecord>,
    pub final_runtime_snapshot: RuntimeSnapshot,
    pub final_rpg_snapshot: RpgSnapshotV2,
    pub final_physics_snapshot: next_contracts::PhysicsCanonicalSnapshotV2,
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
    WorldCheckpoint(next_contracts::WorldCheckpointError),
    SnapshotCanonicalization(CanonicalError),
    StateRoot(StateRootError),
    InitialSnapshotMismatch {
        expected_state_root: StateRoot,
        actual_state_root: StateRoot,
    },
    ComparePointMismatch(Box<ReplayComparePointMismatch>),
    RecordedStageMismatch {
        tick: u64,
        stage: &'static str,
    },
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
            Self::Manifest(ManifestValidationError::UnsupportedReplayVersion(_)) => {
                "UNSUPPORTED_REPLAY_MANIFEST_VERSION"
            }
            Self::Manifest(ManifestValidationError::RpgSchemaUnsupported) => {
                "RPG_SCHEMA_UNSUPPORTED"
            }
            Self::Manifest(_) => "REPLAY_MANIFEST_INVALID",
            Self::SnapshotRestore(_) => "REPLAY_SNAPSHOT_RESTORE_FAILED",
            Self::WorldCheckpoint(_) => "REPLAY_WORLD_CHECKPOINT_FAILED",
            Self::SnapshotCanonicalization(_) => "REPLAY_SNAPSHOT_CANONICALIZATION_FAILED",
            Self::StateRoot(_) => "REPLAY_STATE_ROOT_FAILED",
            Self::InitialSnapshotMismatch { .. } => "REPLAY_INITIAL_SNAPSHOT_MISMATCH",
            Self::ComparePointMismatch { .. } => "NONDETERMINISTIC_RESULT",
            Self::RecordedStageMismatch { .. } => "NONDETERMINISTIC_RESULT",
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
            Self::WorldCheckpoint(error) => {
                write!(formatter, "replay world checkpoint failed: {error}")
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
            Self::RecordedStageMismatch { tick, stage } => {
                write!(
                    formatter,
                    "NONDETERMINISTIC_RESULT at tick {tick}, stage {stage}"
                )
            }
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

impl From<next_contracts::WorldCheckpointError> for ReplayError {
    fn from(error: next_contracts::WorldCheckpointError) -> Self {
        Self::WorldCheckpoint(error)
    }
}

impl From<StateRootError> for ReplayError {
    fn from(error: StateRootError) -> Self {
        Self::StateRoot(error)
    }
}

pub fn run_replay(input: &ReplayInput) -> Result<ReplayOutput, ReplayError> {
    let mut runtime = RuntimeState::new(input.bootstrap.clone(), input.authority.clone())?;
    let mut records = Vec::with_capacity(input.ticks.len());
    for tick in &input.ticks {
        let report = runtime.run_tick(tick.commands.clone())?;
        let checkpoint = runtime.world_checkpoint()?;
        let state_root = compute_world_checkpoint_root(&checkpoint)?;
        let command_ledger_hash = report.snapshot.command_ledger_hash()?;
        records.push(ReplayTickRecord {
            tick: report.tick,
            command_results: report.results,
            events: report.events,
            state_root,
            command_ledger_hash,
        });
    }
    let final_checkpoint = runtime.world_checkpoint()?;
    Ok(ReplayOutput {
        ticks: records,
        final_snapshot: final_checkpoint.runtime_snapshot.clone(),
        final_checkpoint,
    })
}

pub fn run_rpg_replay(input: &RpgReplayInput) -> Result<RpgReplayOutput, ReplayError> {
    let mut runtime = RuntimeState::with_rpg_snapshot(
        input.bootstrap.clone(),
        input.authority.clone(),
        input.initial_rpg_snapshot.clone(),
    )?;
    let mut records = Vec::with_capacity(input.ticks.len());
    for tick in &input.ticks {
        let report = runtime.run_tick(tick.commands.clone())?;
        let checkpoint = runtime.world_checkpoint()?;
        let state_root = compute_world_checkpoint_root(&checkpoint)?;
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
        final_physics_snapshot: runtime.physics_snapshot().clone(),
    })
}

pub fn run_replay_manifest(manifest: &ReplayManifestV4) -> Result<ReplayOutput, ReplayError> {
    run_replay_manifest_with_physics_options(
        manifest,
        next_runtime::PhysicsLaunchOptions::default(),
    )
}

fn run_replay_manifest_with_physics_options(
    manifest: &ReplayManifestV4,
    physics_options: next_runtime::PhysicsLaunchOptions,
) -> Result<ReplayOutput, ReplayError> {
    run_replay_manifest_with_definitions_and_physics_options(
        manifest,
        next_contracts::RpgDefinitionRegistryV1::empty()
            .expect("empty RPG definition registry is canonical"),
        physics_options,
    )
}

pub(crate) fn run_replay_manifest_with_definitions_and_physics_options(
    manifest: &ReplayManifestV4,
    rpg_definitions: next_contracts::RpgDefinitionRegistryV1,
    physics_options: next_runtime::PhysicsLaunchOptions,
) -> Result<ReplayOutput, ReplayError> {
    let limits = CanonicalDecodeLimits::default();
    let (initial_checkpoint, decoded_ticks) = manifest.validate_and_decode(limits)?;

    let actual_initial_root = compute_world_checkpoint_root(&initial_checkpoint)?;
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
    let mut replay = RuntimeReplayDriver::new_with_definitions_and_physics_options(
        initial_checkpoint,
        authority,
        rpg_definitions,
        physics_options,
    )?;
    let mut records = Vec::with_capacity(decoded_ticks.len());
    for ((tick_manifest, tick), compare_point) in manifest
        .ticks
        .iter()
        .zip(decoded_ticks)
        .zip(&manifest.compare_points)
    {
        let closed_ingress_batch = tick.closed_ingress_batch;
        let direct_external_commands = tick.direct_external_commands;
        let expected_ingress_command_batch = tick.expected_ingress_command_batch;
        let expected_physics_step_input = tick.expected_physics_step_input;
        let expected_contact_batch = tick.expected_contact_batch;
        let expected_outcome_command_batch = tick.expected_outcome_command_batch;
        let report = match replay.replay_tick(
            closed_ingress_batch,
            direct_external_commands,
            &expected_ingress_command_batch,
            &expected_physics_step_input,
            &expected_contact_batch,
            &expected_outcome_command_batch,
        ) {
            Ok(report) => report,
            Err(RuntimeReplayError::Runtime(error)) => return Err(error.into()),
            Err(
                RuntimeReplayError::CommandBatchMismatch { .. }
                | RuntimeReplayError::PhysicsStepInputMismatch { .. }
                | RuntimeReplayError::ContactBatchMismatch { .. },
            ) => {
                return Err(ReplayError::RecordedStageMismatch {
                    tick: tick_manifest.tick,
                    stage: "command-batch-before-execution",
                });
            }
            Err(_) => {
                return Err(ReplayError::RecordedStageMismatch {
                    tick: tick_manifest.tick,
                    stage: "replay-driver",
                });
            }
        };
        if report.mapping_receipts != tick.expected_mapping_receipts
            || replay_command_results(&report.results) != tick.expected_command_results
            || report.events != tick.expected_events
        {
            return Err(ReplayError::RecordedStageMismatch {
                tick: tick_manifest.tick,
                stage: "closed-ingress-command-outcome",
            });
        }
        let checkpoint = replay
            .world_checkpoint()
            .map_err(SnapshotRestoreError::from)?;
        let state_root = compute_world_checkpoint_root(&checkpoint)?;
        let command_ledger_hash = report.snapshot.command_ledger_hash()?;
        let (runtime_segment_hash, rpg_segment_hash, physics_segment_hash) =
            checkpoint_segment_hashes(&checkpoint)?;
        if state_root != compare_point.state_root
            || command_ledger_hash != compare_point.command_ledger_hash
            || runtime_segment_hash != compare_point.runtime_segment_hash
            || rpg_segment_hash != compare_point.rpg_segment_hash
            || physics_segment_hash != compare_point.physics_segment_hash
            || report.physics_step_input.input_hash()? != compare_point.physics_step_input_hash
            || report.contact_batch.batch_hash != compare_point.contact_batch_hash
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
    let final_checkpoint = replay.world_checkpoint()?;
    Ok(ReplayOutput {
        ticks: records,
        final_snapshot: final_checkpoint.runtime_snapshot.clone(),
        final_checkpoint,
    })
}

pub fn compute_world_checkpoint_root(
    checkpoint: &WorldCheckpointV4,
) -> Result<StateRoot, ReplayError> {
    checkpoint.validate()?;
    Ok(checkpoint.state_root)
}

pub(crate) fn checkpoint_segment_hashes(
    checkpoint: &WorldCheckpointV4,
) -> Result<
    (
        next_contracts::ContentHash,
        next_contracts::ContentHash,
        next_contracts::ContentHash,
    ),
    ReplayError,
> {
    let runtime = SaveSegmentDescriptor::for_bytes(
        SchemaId::new(RUNTIME_SNAPSHOT_OWNER_ID).map_err(CanonicalError::InvalidIdentifier)?,
        SchemaId::new(RUNTIME_SNAPSHOT_SCHEMA_ID).map_err(CanonicalError::InvalidIdentifier)?,
        SchemaId::new(RUNTIME_SNAPSHOT_SEGMENT_ID).map_err(CanonicalError::InvalidIdentifier)?,
        next_contracts::RUNTIME_SNAPSHOT_SCHEMA_VERSION,
        &checkpoint.runtime_snapshot.canonical_bytes()?,
    )?;
    let rpg = SaveSegmentDescriptor::for_bytes(
        SchemaId::new(RPG_AGGREGATE_SNAPSHOT_OWNER_ID)
            .map_err(CanonicalError::InvalidIdentifier)?,
        SchemaId::new(RPG_AGGREGATE_SNAPSHOT_SCHEMA_ID)
            .map_err(CanonicalError::InvalidIdentifier)?,
        SchemaId::new(RPG_AGGREGATE_SNAPSHOT_SEGMENT_ID)
            .map_err(CanonicalError::InvalidIdentifier)?,
        next_contracts::RPG_AGGREGATE_SNAPSHOT_SCHEMA_VERSION,
        &checkpoint.rpg_snapshot.canonical_bytes()?,
    )?;
    let physics = SaveSegmentDescriptor::for_bytes(
        SchemaId::new(PHYSICS_SNAPSHOT_OWNER_ID).map_err(CanonicalError::InvalidIdentifier)?,
        SchemaId::new(PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID)
            .map_err(CanonicalError::InvalidIdentifier)?,
        SchemaId::new(PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID)
            .map_err(CanonicalError::InvalidIdentifier)?,
        u32::from(PHYSICS_WORLD_CHECKPOINT_SCHEMA_VERSION),
        &checkpoint.physics_checkpoint.canonical_bytes()?,
    )?;
    Ok((runtime.content_hash, rpg.content_hash, physics.content_hash))
}

pub(crate) fn replay_command_results(results: &[CommandResult]) -> Vec<ReplayCommandResultV2> {
    results
        .iter()
        .map(|result| ReplayCommandResultV2 {
            command_id: result.command_id,
            sequence: result.sequence,
            disposition_code: match result.disposition {
                next_runtime::CommandDisposition::Reserved => "RESERVED".to_owned(),
                next_runtime::CommandDisposition::Committed => "COMMITTED".to_owned(),
                next_runtime::CommandDisposition::Deduplicated => "DEDUPLICATED".to_owned(),
                next_runtime::CommandDisposition::Rejected(code) => code.as_str().to_owned(),
            },
        })
        .collect()
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
    if expected.final_snapshot != actual.final_snapshot
        || expected.final_checkpoint != actual.final_checkpoint
    {
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
mod tests;
