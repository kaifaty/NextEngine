use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::canonical::{CanonicalDecodeLimits, CanonicalError};
use next_contracts::command::{DomainEvent, WorldCommand};
use next_contracts::ids::{CommandLedgerHash, SchemaId, StateRoot};
use next_contracts::persistence::{
    ManifestValidationError, ReplayCommandResultV2, ReplayManifestV7, SaveSegmentDescriptor,
    WorldStreamingReplayInputV1, replay_physics_query_batch_hash,
    replay_physics_query_results_hash, replay_targeting_query_trace_hash,
};
use next_contracts::physics::{
    PHYSICS_SNAPSHOT_OWNER_ID, PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID,
    PHYSICS_WORLD_CHECKPOINT_SCHEMA_VERSION, PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID,
};
use next_contracts::rpg::{
    RPG_AGGREGATE_SNAPSHOT_OWNER_ID, RPG_AGGREGATE_SNAPSHOT_SCHEMA_ID,
    RPG_AGGREGATE_SNAPSHOT_SEGMENT_ID, RpgSnapshotV2,
};
use next_contracts::snapshot::{
    RUNTIME_SNAPSHOT_OWNER_ID, RUNTIME_SNAPSHOT_SCHEMA_ID, RUNTIME_SNAPSHOT_SEGMENT_ID,
    RuntimeSnapshotV3, WorldCheckpointV4,
};
use next_runtime::{
    AuthorityRegistry, CommandResult, RuntimeBootstrapV4, RuntimeFatalError, RuntimeReplayDriver,
    RuntimeReplayError, RuntimeState, SnapshotRestoreError,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayInput {
    pub bootstrap: RuntimeBootstrapV4,
    pub authority: AuthorityRegistry,
    pub ticks: Vec<ReplayTickInput>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RpgReplayInput {
    pub bootstrap: RuntimeBootstrapV4,
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
    pub final_snapshot: RuntimeSnapshotV3,
    pub final_checkpoint: WorldCheckpointV4,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RpgReplayOutput {
    pub ticks: Vec<ReplayTickRecord>,
    pub final_runtime_snapshot: RuntimeSnapshotV3,
    pub final_rpg_snapshot: RpgSnapshotV2,
    pub final_physics_snapshot: next_contracts::physics::PhysicsCanonicalSnapshotV2,
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
    WorldStreaming(next_world::WorldStreamingError),
    WorldStreamingContract(next_contracts::world::WorldStreamingContractError),
    WorldRoutine(next_world::WorldRoutineOwnerError),
    WorldPopulation(next_world::WorldPopulationOwnerError),
    Manifest(ManifestValidationError),
    SnapshotRestore(SnapshotRestoreError),
    WorldCheckpoint(next_contracts::snapshot::WorldCheckpointError),
    SnapshotCanonicalization(CanonicalError),
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
            Self::WorldStreaming(_) => "REPLAY_WORLD_STREAMING_FAILED",
            Self::WorldStreamingContract(_) => "REPLAY_WORLD_STREAMING_FAILED",
            Self::WorldRoutine(_) => "REPLAY_WORLD_ROUTINE_FAILED",
            Self::WorldPopulation(_) => "REPLAY_WORLD_POPULATION_FAILED",
            Self::Manifest(ManifestValidationError::UnsupportedReplayVersion(_)) => {
                "UNSUPPORTED_REPLAY_MANIFEST_VERSION"
            }
            Self::Manifest(_) => "REPLAY_MANIFEST_INVALID",
            Self::SnapshotRestore(_) => "REPLAY_SNAPSHOT_RESTORE_FAILED",
            Self::WorldCheckpoint(_) => "REPLAY_WORLD_CHECKPOINT_FAILED",
            Self::SnapshotCanonicalization(_) => "REPLAY_SNAPSHOT_CANONICALIZATION_FAILED",
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
            Self::WorldStreaming(error) => {
                write!(formatter, "world streaming failed during replay: {error}")
            }
            Self::WorldStreamingContract(error) => {
                write!(
                    formatter,
                    "world streaming contract failed during replay: {error}"
                )
            }
            Self::WorldRoutine(error) => {
                write!(formatter, "world routine failed during replay: {error}")
            }
            Self::WorldPopulation(error) => {
                write!(formatter, "world population failed during replay: {error}")
            }
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

impl From<next_world::WorldStreamingError> for ReplayError {
    fn from(error: next_world::WorldStreamingError) -> Self {
        Self::WorldStreaming(error)
    }
}

impl From<next_contracts::world::WorldStreamingContractError> for ReplayError {
    fn from(error: next_contracts::world::WorldStreamingContractError) -> Self {
        Self::WorldStreamingContract(error)
    }
}

impl From<next_world::WorldRoutineOwnerError> for ReplayError {
    fn from(error: next_world::WorldRoutineOwnerError) -> Self {
        Self::WorldRoutine(error)
    }
}

impl From<next_world::WorldPopulationOwnerError> for ReplayError {
    fn from(error: next_world::WorldPopulationOwnerError) -> Self {
        Self::WorldPopulation(error)
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

impl From<next_contracts::snapshot::WorldCheckpointError> for ReplayError {
    fn from(error: next_contracts::snapshot::WorldCheckpointError) -> Self {
        Self::WorldCheckpoint(error)
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

pub fn run_replay_manifest_v7(
    manifest: &ReplayManifestV7,
    package: next_project::ActivatedProjectPackage,
) -> Result<ReplayOutput, ReplayError> {
    run_replay_manifest_v7_with_physics_options(
        manifest,
        package,
        next_runtime::PhysicsLaunchOptions::default(),
    )
}

pub fn run_replay_manifest_v7_with_physics_options(
    manifest: &ReplayManifestV7,
    package: next_project::ActivatedProjectPackage,
    physics_options: next_runtime::PhysicsLaunchOptions,
) -> Result<ReplayOutput, ReplayError> {
    let limits = CanonicalDecodeLimits::default();
    let (initial, decoded_ticks) = manifest.validate_and_decode(limits)?;
    let next_project::ActivatedProjectPackage {
        project,
        content_generation,
    } = package;
    let mut world = next_world::WorldStreamerV1::restore(
        project.clone(),
        content_generation,
        initial.world_streaming_snapshot,
    )?;
    let mut routine = next_world::WorldRoutineOwnerV1::restore(
        project.world_routine_catalog_or_none,
        initial.world_routine_snapshot_or_none,
        initial.checkpoint.runtime_snapshot.next_tick,
    )?;
    let mut population = next_world::WorldPopulationOwnerV1::restore(
        project.world_population_catalog.clone(),
        project.world_navigation_catalog.clone(),
        initial.world_population_snapshot,
        initial.checkpoint.runtime_snapshot.next_tick,
    )?;

    let mut authority = AuthorityRegistry::new();
    for grant in &manifest.authority {
        authority
            .register(grant.principal.clone(), grant.capabilities.clone())
            .map_err(|_| ManifestValidationError::AuthorityNotStrictlySorted)?;
    }
    let mut replay = RuntimeReplayDriver::new_with_definitions_and_physics_options(
        initial.checkpoint,
        authority,
        project.rpg_definitions.clone(),
        physics_options,
    )?;
    replay.validate_world_routine_ledger_closure(&routine)?;
    replay.validate_world_population_ledger_closure(&population)?;
    let mut records = Vec::with_capacity(decoded_ticks.len());
    for ((tick_manifest, tick), compare_point) in manifest
        .ticks
        .iter()
        .zip(decoded_ticks)
        .zip(&manifest.compare_points)
    {
        let streaming = prepare_replay_world_streaming_input(
            &tick.world_streaming_input,
            tick_manifest.tick,
            &mut world,
        )?;
        let commit = match replay.replay_world_services_tick_v7(
            &mut routine,
            &mut population,
            &mut world,
            streaming,
            tick.closed_ingress_batch,
            tick.direct_external_commands,
            &tick.expected_ingress_command_batch,
            &tick.expected_physics_step_input,
            &tick.expected_contact_batch,
            &tick.expected_targeting_intents,
            &tick.expected_authoritative_targeting_queries,
            &tick.expected_physics_query_batch,
            &tick.expected_physics_query_results,
            &tick.expected_outcome_command_batch,
            &tick.expected_interaction_availability,
        ) {
            Ok(commit) => commit,
            Err(RuntimeReplayError::Runtime(error)) => return Err(error.into()),
            Err(
                RuntimeReplayError::CommandBatchMismatch { .. }
                | RuntimeReplayError::PhysicsStepInputMismatch { .. }
                | RuntimeReplayError::ContactBatchMismatch { .. }
                | RuntimeReplayError::TargetingIntentMismatch { .. }
                | RuntimeReplayError::TargetingQueryMismatch { .. }
                | RuntimeReplayError::PhysicsQueryBatchMismatch { .. }
                | RuntimeReplayError::PhysicsQueryResultMismatch { .. }
                | RuntimeReplayError::InteractionAvailabilityMismatch { .. },
            ) => {
                return Err(ReplayError::RecordedStageMismatch {
                    tick: tick_manifest.tick,
                    stage: "closed-authoritative-query-outcome",
                });
            }
            Err(_) => {
                return Err(ReplayError::RecordedStageMismatch {
                    tick: tick_manifest.tick,
                    stage: "replay-driver",
                });
            }
        };
        let state_root = commit.application_state_root;
        let owner_segments = commit.application_owner_segments.clone();
        let report = commit.runtime_report;
        if report.mapping_receipts != tick.expected_mapping_receipts
            || report.interaction_availability != tick.expected_interaction_availability
            || replay_command_results(&report.results) != tick.expected_command_results
            || report.events != tick.expected_events
        {
            return Err(ReplayError::RecordedStageMismatch {
                tick: tick_manifest.tick,
                stage: "closed-ingress-command-outcome",
            });
        }
        let command_ledger_hash = report.snapshot.command_ledger_hash()?;
        if state_root != compare_point.state_root
            || command_ledger_hash != compare_point.command_ledger_hash
            || owner_segments != compare_point.owner_segments
            || report.closed_ingress_batch.batch_hash != compare_point.closed_ingress_batch_hash
            || report.command_batches[0].batch_hash != compare_point.ingress_command_batch_hash
            || report.physics_step_input.input_hash()? != compare_point.physics_step_input_hash
            || report.contact_batch.batch_hash != compare_point.contact_batch_hash
            || replay_physics_query_batch_hash(&report.physics_query_batch)?
                != compare_point.physics_query_batch_hash
            || replay_physics_query_results_hash(&report.physics_query_results)?
                != compare_point.physics_query_results_hash
            || replay_targeting_query_trace_hash(
                &report.targeting_intents,
                &report.authoritative_targeting_queries,
            )? != compare_point.targeting_query_trace_hash
            || report.command_batches[1].batch_hash != compare_point.outcome_command_batch_hash
            || next_contracts::world_routine::interaction_availability_batch_hash(
                &report.interaction_availability,
            )
            .map_err(ManifestValidationError::from)?
                != compare_point.interaction_availability_hash
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

fn prepare_replay_world_streaming_input(
    input: &WorldStreamingReplayInputV1,
    tick: u64,
    world: &mut next_world::WorldStreamerV1,
) -> Result<Option<next_world::PreparedWorldStreamingPublicationV1>, ReplayError> {
    let base_hash = world.snapshot().state_hash()?;
    let mismatch = || ReplayError::RecordedStageMismatch {
        tick,
        stage: "world-streaming-input",
    };
    match input {
        WorldStreamingReplayInputV1::None => Ok(None),
        WorldStreamingReplayInputV1::BeginTransition {
            target_chunk_id,
            expected_base_world_state_hash,
            expected_next_world_state_hash,
        } => {
            if base_hash != *expected_base_world_state_hash {
                return Err(mismatch());
            }
            let publication = world.prepare_begin_transition(target_chunk_id.clone(), tick)?;
            if publication.next_world_state_hash() != *expected_next_world_state_hash {
                return Err(mismatch());
            }
            Ok(Some(publication))
        }
        WorldStreamingReplayInputV1::CompletePendingTransition {
            target_chunk_id,
            expected_base_world_state_hash,
            expected_loaded_result_hash,
            expected_next_world_state_hash,
        } => {
            if base_hash != *expected_base_world_state_hash
                || world
                    .snapshot()
                    .pending_transition
                    .as_ref()
                    .is_none_or(|pending| &pending.target_chunk_id != target_chunk_id)
            {
                return Err(mismatch());
            }
            let loaded = world.load_pending(next_world::WORLD_CHUNK_DEFAULT_WORKERS)?;
            if loaded.result_hash() != *expected_loaded_result_hash {
                return Err(mismatch());
            }
            let publication = world.prepare_loaded_commit(loaded, tick)?;
            if publication.next_world_state_hash() != *expected_next_world_state_hash {
                return Err(mismatch());
            }
            Ok(Some(publication))
        }
    }
}

pub fn compute_world_checkpoint_root(
    checkpoint: &WorldCheckpointV4,
) -> Result<StateRoot, ReplayError> {
    checkpoint.validate()?;
    Ok(checkpoint.state_root)
}

pub fn checkpoint_segment_hashes(
    checkpoint: &WorldCheckpointV4,
) -> Result<
    (
        next_contracts::ids::ContentHash,
        next_contracts::ids::ContentHash,
        next_contracts::ids::ContentHash,
    ),
    ReplayError,
> {
    let runtime = SaveSegmentDescriptor::for_bytes(
        SchemaId::new(RUNTIME_SNAPSHOT_OWNER_ID).map_err(CanonicalError::InvalidIdentifier)?,
        SchemaId::new(RUNTIME_SNAPSHOT_SCHEMA_ID).map_err(CanonicalError::InvalidIdentifier)?,
        SchemaId::new(RUNTIME_SNAPSHOT_SEGMENT_ID).map_err(CanonicalError::InvalidIdentifier)?,
        next_contracts::snapshot::RUNTIME_SNAPSHOT_SCHEMA_VERSION,
        &checkpoint.runtime_snapshot.canonical_bytes()?,
    )?;
    let rpg = SaveSegmentDescriptor::for_bytes(
        SchemaId::new(RPG_AGGREGATE_SNAPSHOT_OWNER_ID)
            .map_err(CanonicalError::InvalidIdentifier)?,
        SchemaId::new(RPG_AGGREGATE_SNAPSHOT_SCHEMA_ID)
            .map_err(CanonicalError::InvalidIdentifier)?,
        SchemaId::new(RPG_AGGREGATE_SNAPSHOT_SEGMENT_ID)
            .map_err(CanonicalError::InvalidIdentifier)?,
        next_contracts::rpg::RPG_AGGREGATE_SNAPSHOT_SCHEMA_VERSION,
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

pub fn replay_command_results(results: &[CommandResult]) -> Vec<ReplayCommandResultV2> {
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
