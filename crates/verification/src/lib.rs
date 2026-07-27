#![forbid(unsafe_code)]

mod persistence_replay;
mod physics_parity;
mod player_fixture;

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::{
    CanonicalDecodeLimits, CanonicalError, CapabilityId, CommandLedgerHash, CommandStreamId,
    CommandStreamRegistryV1, DomainEvent, IssuerPrincipal, ManifestValidationError,
    PHYSICS_SNAPSHOT_OWNER_ID, PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID,
    PHYSICS_WORLD_CHECKPOINT_SCHEMA_VERSION, PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID,
    PrincipalRecordV1, PrincipalRegistryV1, PrincipalStatus, ProjectId, RPG_SNAPSHOT_OWNER_ID,
    RPG_SNAPSHOT_SCHEMA_ID, RPG_SNAPSHOT_SEGMENT_ID, RUNTIME_SNAPSHOT_OWNER_ID,
    RUNTIME_SNAPSHOT_SCHEMA_ID, RUNTIME_SNAPSHOT_SEGMENT_ID, ReplayCommandResultV2,
    ReplayManifestV3, RpgSnapshot, RuntimeDeterminismProfileV1, RuntimeSnapshot,
    SaveSegmentDescriptor, SchemaId, StateRoot, WorldCheckpointV3, WorldCommand,
    WorldIdentityManifestV1, content_hash_from_bytes, sha256,
};
use next_runtime::{
    AuthorityRegistry, AuthorityRegistryError, CommandKindRegistry, CommandResult,
    RuntimeBootstrapV3, RuntimeFatalError, RuntimeReplayDriver, RuntimeReplayError, RuntimeState,
    SnapshotRestoreError,
};

pub use persistence_replay::{
    PersistenceReplayBackend, PersistenceReplayCheckError, PersistenceReplayCheckReport,
    run_persistence_replay_check, run_persistence_replay_check_with_backend,
};
pub use physics_parity::{
    PhysicsBackendParityError, PhysicsBackendParityReport, run_physics_backend_parity_check,
};
pub use player_fixture::{
    CanonicalFixtureError, NeutralPlayerFixture, PhysicsCollisionBackend,
    PhysicsCollisionCheckReport, PlayCheckError, PlayCheckReport, build_neutral_player_fixture,
    build_physx_player_fixture, core_interaction_rpg_snapshot, player_action_sample,
    player_interact_sample, run_physics_collision_check, run_physics_collision_check_with_backend,
    run_play_check,
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
    pub bootstrap: RuntimeBootstrapV3,
    pub authority: AuthorityRegistry,
    pub ticks: Vec<ReplayTickInput>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RpgReplayInput {
    pub bootstrap: RuntimeBootstrapV3,
    pub authority: AuthorityRegistry,
    pub initial_rpg_snapshot: RpgSnapshot,
    pub ticks: Vec<ReplayTickInput>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayTickInput {
    pub commands: Vec<WorldCommand>,
}

#[derive(Clone, Debug)]
pub struct NeutralRuntimeFixture {
    pub bootstrap: RuntimeBootstrapV3,
    pub authority: AuthorityRegistry,
    pub streams: BTreeMap<IssuerPrincipal, CommandStreamId>,
}

impl NeutralRuntimeFixture {
    #[must_use]
    pub fn stream_for(&self, principal: &IssuerPrincipal) -> Option<CommandStreamId> {
        self.streams.get(principal).copied()
    }
}

pub fn build_neutral_runtime_fixture(
    project_id: &str,
    grants: impl IntoIterator<Item = (IssuerPrincipal, Vec<CapabilityId>)>,
) -> Result<NeutralRuntimeFixture, NeutralFixtureError> {
    let registry_hash = CommandKindRegistry::core_v1().canonical_hash();
    let profile = RuntimeDeterminismProfileV1::bootstrap_default(registry_hash);
    let world_identity = WorldIdentityManifestV1::new(
        ProjectId::new(project_id)?,
        sha256(format!("nextengine.fixture.nonce:{project_id}").as_bytes()),
        sha256(format!("nextengine.fixture.rng:{project_id}").as_bytes()),
        profile.profile_hash()?,
    )?;
    let mut principal_registry = PrincipalRegistryV1::empty(world_identity.world_namespace);
    let mut stream_registry = CommandStreamRegistryV1::empty(world_identity.world_namespace);
    let mut authority = AuthorityRegistry::new();
    let mut streams = BTreeMap::new();
    let mut grants: Vec<_> = grants.into_iter().collect();
    grants.sort_by(|left, right| left.0.cmp(&right.0));
    if grants.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(NeutralFixtureError::DuplicatePrincipal);
    }
    for (principal, capabilities) in grants {
        let principal_bytes = principal.canonical_bytes()?;
        let mut provenance = Vec::new();
        provenance.extend_from_slice(world_identity.world_namespace.as_bytes());
        provenance.extend_from_slice(&(principal_bytes.len() as u64).to_le_bytes());
        provenance.extend_from_slice(&principal_bytes);
        let provenance_hash = content_hash_from_bytes(sha256(&provenance));
        principal_registry.register(
            principal.clone(),
            PrincipalRecordV1 {
                provenance_hash,
                capability_subject_id: SchemaId::new(format!(
                    "fixture.principal.{}.{}",
                    principal.tag(),
                    hex_identifier(principal.identifier_bytes())
                ))?,
                status: PrincipalStatus::Active,
            },
        )?;
        let stream_id = stream_registry.allocate_stream(principal.clone())?;
        authority.register(principal.clone(), capabilities)?;
        streams.insert(principal, stream_id);
    }
    Ok(NeutralRuntimeFixture {
        bootstrap: RuntimeBootstrapV3::new(
            world_identity,
            principal_registry,
            stream_registry,
            profile,
        ),
        authority,
        streams,
    })
}

#[derive(Debug)]
pub enum NeutralFixtureError {
    Canonical(CanonicalError),
    Identifier(next_contracts::IdentifierError),
    Identity(next_contracts::IdentityContractError),
    Physics(next_contracts::PhysicsContractError),
    Authority(AuthorityRegistryError),
    DuplicatePrincipal,
}

impl Display for NeutralFixtureError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Canonical(error) => write!(formatter, "fixture canonicalization failed: {error}"),
            Self::Identifier(error) => write!(formatter, "fixture identifier failed: {error}"),
            Self::Identity(error) => write!(formatter, "fixture identity failed: {error}"),
            Self::Physics(error) => write!(formatter, "fixture physics failed: {error}"),
            Self::Authority(error) => write!(formatter, "fixture authority failed: {error}"),
            Self::DuplicatePrincipal => formatter.write_str("fixture principal is duplicated"),
        }
    }
}

impl Error for NeutralFixtureError {}

impl From<CanonicalError> for NeutralFixtureError {
    fn from(error: CanonicalError) -> Self {
        Self::Canonical(error)
    }
}

impl From<next_contracts::IdentifierError> for NeutralFixtureError {
    fn from(error: next_contracts::IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

impl From<next_contracts::IdentityContractError> for NeutralFixtureError {
    fn from(error: next_contracts::IdentityContractError) -> Self {
        Self::Identity(error)
    }
}

impl From<next_contracts::PhysicsContractError> for NeutralFixtureError {
    fn from(error: next_contracts::PhysicsContractError) -> Self {
        Self::Physics(error)
    }
}

impl From<AuthorityRegistryError> for NeutralFixtureError {
    fn from(error: AuthorityRegistryError) -> Self {
        Self::Authority(error)
    }
}

fn hex_identifier(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        value.push(char::from(DIGITS[usize::from(byte >> 4)]));
        value.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    value
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayOutput {
    pub ticks: Vec<ReplayTickRecord>,
    pub final_snapshot: RuntimeSnapshot,
    pub final_checkpoint: WorldCheckpointV3,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RpgReplayOutput {
    pub ticks: Vec<ReplayTickRecord>,
    pub final_runtime_snapshot: RuntimeSnapshot,
    pub final_rpg_snapshot: RpgSnapshot,
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

pub fn run_replay_manifest(manifest: &ReplayManifestV3) -> Result<ReplayOutput, ReplayError> {
    run_replay_manifest_with_physics_options(
        manifest,
        next_runtime::PhysicsLaunchOptions::default(),
    )
}

pub(crate) fn run_replay_manifest_with_physics_options(
    manifest: &ReplayManifestV3,
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
    let mut replay = RuntimeReplayDriver::new_with_physics_options(
        initial_checkpoint,
        authority,
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
    checkpoint: &WorldCheckpointV3,
) -> Result<StateRoot, ReplayError> {
    checkpoint.validate()?;
    Ok(checkpoint.state_root)
}

fn checkpoint_segment_hashes(
    checkpoint: &WorldCheckpointV3,
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
        SchemaId::new(RPG_SNAPSHOT_OWNER_ID).map_err(CanonicalError::InvalidIdentifier)?,
        SchemaId::new(RPG_SNAPSHOT_SCHEMA_ID).map_err(CanonicalError::InvalidIdentifier)?,
        SchemaId::new(RPG_SNAPSHOT_SEGMENT_ID).map_err(CanonicalError::InvalidIdentifier)?,
        next_contracts::RPG_SNAPSHOT_SCHEMA_VERSION,
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

fn replay_command_results(results: &[CommandResult]) -> Vec<ReplayCommandResultV2> {
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
mod tests {
    use next_contracts::{
        AuthorityGrant, CanonicalDecodeLimits, CapabilityId, ContentHash, IssuerPrincipal,
        ManifestCodecError, ManifestValidationError, NOOP_COMMAND_CAPABILITY_ID,
        PHYSICS_SNAPSHOT_OWNER_ID, PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID,
        PHYSICS_WORLD_CHECKPOINT_SCHEMA_VERSION, PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID,
        PhysicsWorldCheckpointV1, PlayerPrincipalId, REPLAY_MANIFEST_V3_SCHEMA_VERSION,
        RPG_SNAPSHOT_OWNER_ID, RPG_SNAPSHOT_SCHEMA_ID, RPG_SNAPSHOT_SCHEMA_VERSION,
        RPG_SNAPSHOT_SEGMENT_ID, RUNTIME_SNAPSHOT_OWNER_ID, RUNTIME_SNAPSHOT_SCHEMA_ID,
        RUNTIME_SNAPSHOT_SCHEMA_VERSION, RUNTIME_SNAPSHOT_SEGMENT_ID, ReplayCommandRecord,
        ReplayComparePointV3, ReplayManifestV3, ReplayOwnerSegmentV2, ReplayTickManifestV3,
        SaveCompatibility, SaveSegmentDescriptor, SchemaId, StateRoot, TickSettings,
        WorldCheckpointV3, WorldCommand,
    };
    use next_runtime::{AuthorityRegistry, RuntimeReplayDriver, RuntimeReplayError, RuntimeState};

    use super::{
        ReplayError, ReplayInput, ReplayTickInput, StateSegment, build_neutral_runtime_fixture,
        checkpoint_segment_hashes, compare_replay_outputs, compute_state_root,
        compute_world_checkpoint_root, replay_command_results, run_replay, run_replay_manifest,
        verify_replay,
    };

    fn command(
        stream: next_contracts::CommandStreamId,
        issuer: IssuerPrincipal,
        sequence: u64,
        tick: u64,
    ) -> WorldCommand {
        WorldCommand::noop(stream, issuer, sequence, tick).expect("test command is canonical")
    }

    fn scenario() -> ReplayInput {
        let first = IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([2; 16]));
        let second = IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([1; 16]));
        let capability =
            CapabilityId::new(NOOP_COMMAND_CAPABILITY_ID).expect("built-in capability ID is valid");
        let fixture = build_neutral_runtime_fixture(
            "nextengine.replay-test",
            [
                (first.clone(), vec![capability.clone()]),
                (second.clone(), vec![capability]),
            ],
        )
        .expect("fixture");
        let first_stream = fixture.stream_for(&first).expect("first stream");
        let second_stream = fixture.stream_for(&second).expect("second stream");
        ReplayInput {
            bootstrap: fixture.bootstrap,
            authority: fixture.authority,
            ticks: vec![
                ReplayTickInput {
                    commands: vec![
                        command(first_stream, first.clone(), 0, 0),
                        command(second_stream, second, 0, 0),
                    ],
                },
                ReplayTickInput {
                    commands: vec![command(first_stream, first, 1, 1)],
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
                physics_hz: 60,
                motor_hz: 60,
            },
            loaded_chunk_revisions: vec![],
            rng_stream_states: vec![],
            physical_bindings: vec![],
            policy_state_schemas: vec![],
            plugin_script_bindings: vec![],
        }
    }

    fn replay_manifest() -> (ReplayManifestV3, super::ReplayOutput) {
        let input = scenario();
        let expected = run_replay(&input).expect("reference replay runs");
        let mut runtime = RuntimeState::new(input.bootstrap.clone(), input.authority.clone())
            .expect("initial runtime");
        let initial_checkpoint = runtime.world_checkpoint().expect("initial checkpoint");
        let authority = input
            .authority
            .entries()
            .map(|(principal, capabilities)| AuthorityGrant {
                principal: principal.clone(),
                capabilities: capabilities.iter().cloned().collect(),
            })
            .collect();
        let mut ticks = Vec::new();
        let mut compare_points = Vec::new();
        let physics_catalog = initial_checkpoint.physics_checkpoint.catalog.clone();
        for input_tick in &input.ticks {
            let report = runtime
                .run_tick(input_tick.commands.clone())
                .expect("recorded tick");
            let checkpoint = WorldCheckpointV3::new(
                report.snapshot.clone(),
                report.rpg_snapshot.clone(),
                PhysicsWorldCheckpointV1::new(
                    physics_catalog.clone(),
                    report.physics_snapshot.clone(),
                )
                .expect("physics checkpoint"),
            )
            .expect("record checkpoint");
            let (runtime_segment_hash, rpg_segment_hash, physics_segment_hash) =
                checkpoint_segment_hashes(&checkpoint).expect("segment hashes");
            ticks.push(ReplayTickManifestV3 {
                tick: report.tick,
                closed_ingress_batch: report.closed_ingress_batch.clone(),
                direct_external_commands: input_tick
                    .commands
                    .iter()
                    .map(|command| {
                        ReplayCommandRecord::from_command(command)
                            .expect("test command is canonical")
                    })
                    .collect(),
                expected_ingress_command_batch: report.command_batches[0].clone(),
                expected_physics_step_input: report.physics_step_input.clone(),
                expected_contact_batch: report.contact_batch.clone(),
                expected_outcome_command_batch: report.command_batches[1].clone(),
                expected_mapping_receipts: report.mapping_receipts.clone(),
                expected_command_results: replay_command_results(&report.results),
                expected_events: report.events.clone(),
            });
            compare_points.push(ReplayComparePointV3 {
                tick: report.tick,
                state_root: compute_world_checkpoint_root(&checkpoint).expect("state root"),
                command_ledger_hash: report.snapshot.command_ledger_hash().expect("ledger hash"),
                runtime_segment_hash,
                rpg_segment_hash,
                physics_segment_hash,
                closed_ingress_batch_hash: report.closed_ingress_batch.batch_hash,
                ingress_command_batch_hash: report.command_batches[0].batch_hash,
                physics_step_input_hash: report
                    .physics_step_input
                    .input_hash()
                    .expect("physics input hash"),
                contact_batch_hash: report.contact_batch.batch_hash,
                outcome_command_batch_hash: report.command_batches[1].batch_hash,
            });
        }
        (
            ReplayManifestV3 {
                schema_version: REPLAY_MANIFEST_V3_SCHEMA_VERSION,
                compatibility: compatibility(),
                initial_owner_segments: replay_owner_segments(&initial_checkpoint),
                initial_state_root: compute_world_checkpoint_root(&initial_checkpoint)
                    .expect("initial root computes"),
                authority,
                ticks,
                compare_points,
            },
            expected,
        )
    }

    fn replay_owner_segments(checkpoint: &WorldCheckpointV3) -> Vec<ReplayOwnerSegmentV2> {
        let mut segments = [
            (
                RUNTIME_SNAPSHOT_OWNER_ID,
                RUNTIME_SNAPSHOT_SCHEMA_ID,
                RUNTIME_SNAPSHOT_SEGMENT_ID,
                RUNTIME_SNAPSHOT_SCHEMA_VERSION,
                checkpoint
                    .runtime_snapshot
                    .canonical_bytes()
                    .expect("runtime"),
            ),
            (
                RPG_SNAPSHOT_OWNER_ID,
                RPG_SNAPSHOT_SCHEMA_ID,
                RPG_SNAPSHOT_SEGMENT_ID,
                RPG_SNAPSHOT_SCHEMA_VERSION,
                checkpoint.rpg_snapshot.canonical_bytes().expect("RPG"),
            ),
            (
                PHYSICS_SNAPSHOT_OWNER_ID,
                PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID,
                PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID,
                u32::from(PHYSICS_WORLD_CHECKPOINT_SCHEMA_VERSION),
                checkpoint
                    .physics_checkpoint
                    .canonical_bytes()
                    .expect("physics"),
            ),
        ]
        .into_iter()
        .map(|(owner, schema, segment, version, canonical_bytes)| {
            let descriptor = SaveSegmentDescriptor::for_bytes(
                SchemaId::new(owner).expect("owner"),
                SchemaId::new(schema).expect("schema"),
                SchemaId::new(segment).expect("segment"),
                version,
                &canonical_bytes,
            )
            .expect("descriptor");
            ReplayOwnerSegmentV2 {
                descriptor,
                canonical_bytes,
            }
        })
        .collect::<Vec<_>>();
        segments.sort_by(|left, right| {
            (
                &left.descriptor.owner_id,
                &left.descriptor.schema_id,
                &left.descriptor.segment_id,
            )
                .cmp(&(
                    &right.descriptor.owner_id,
                    &right.descriptor.schema_id,
                    &right.descriptor.segment_id,
                ))
        });
        segments
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
        let prior = second.ticks[1].commands[0].clone();
        second.ticks[1].commands = vec![command(prior.stream_id, prior.issuer.clone(), 2, 1)];
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
    fn replay_manifest_v3_jcs_round_trip_is_byte_exact() {
        let (manifest, _) = replay_manifest();
        let bytes = manifest.to_jcs_bytes().expect("manifest encodes");
        let decoded = ReplayManifestV3::from_jcs_bytes(&bytes, CanonicalDecodeLimits::default())
            .expect("manifest decodes");
        assert_eq!(decoded, manifest);
        assert_eq!(decoded.to_jcs_bytes().expect("manifest re-encodes"), bytes);
    }

    #[test]
    fn replay_manifest_v2_jcs_is_rejected_before_nested_decoding() {
        let bytes = br#"{"schema_version":2}"#;
        assert!(matches!(
            ReplayManifestV3::from_jcs_bytes(bytes, CanonicalDecodeLimits::default()),
            Err(ManifestCodecError::Validation(
                ManifestValidationError::UnsupportedReplayVersion(2)
            ))
        ));
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
        manifest.ticks[1].direct_external_commands[0]
            .canonical_command_bytes
            .push(0);
        let error = run_replay_manifest(&manifest).expect_err("corrupt command must fail closed");
        assert!(matches!(error, ReplayError::Manifest(_)));
        assert_eq!(error.stable_code(), "REPLAY_MANIFEST_INVALID");
    }

    #[test]
    fn replay_driver_rejects_command_batch_before_state_mutation() {
        let (manifest, _) = replay_manifest();
        let (checkpoint, mut ticks) = manifest
            .validate_and_decode(CanonicalDecodeLimits::default())
            .expect("manifest decodes");
        let initial_checkpoint = checkpoint.clone();
        let mut authority = AuthorityRegistry::new();
        for grant in &manifest.authority {
            authority
                .register(grant.principal.clone(), grant.capabilities.clone())
                .expect("authority grant");
        }
        let mut driver =
            RuntimeReplayDriver::new(checkpoint, authority).expect("replay driver restores");
        let tick = ticks.remove(0);
        let mut wrong_ingress_batch = tick.expected_ingress_command_batch;
        wrong_ingress_batch.batch_hash = ContentHash::from_bytes([0xee; 32]);
        let error = driver
            .replay_tick(
                tick.closed_ingress_batch,
                tick.direct_external_commands,
                &wrong_ingress_batch,
                &tick.expected_physics_step_input,
                &tick.expected_contact_batch,
                &tick.expected_outcome_command_batch,
            )
            .expect_err("divergent batch must fail before execution");
        assert!(matches!(
            error,
            RuntimeReplayError::CommandBatchMismatch {
                tick: 0,
                phase: next_contracts::CommandPhase::Ingress,
            }
        ));
        assert_eq!(
            driver.world_checkpoint().expect("driver checkpoint"),
            initial_checkpoint
        );
    }

    #[test]
    fn v2_replay_is_rejected_before_nested_snapshot_decoding() {
        let (mut manifest, _) = replay_manifest();
        manifest.schema_version = 2;
        manifest.initial_owner_segments.clear();
        let error = run_replay_manifest(&manifest).expect_err("V2 replay fails closed");
        assert!(matches!(
            error,
            ReplayError::Manifest(ManifestValidationError::UnsupportedReplayVersion(2))
        ));
        assert_eq!(error.stable_code(), "UNSUPPORTED_REPLAY_MANIFEST_VERSION");
    }

    #[test]
    fn scenario_final_root_pins_v3_checkpoint_identity_ledger_archive_and_physics_closure() {
        assert_eq!(
            run_replay(&scenario())
                .expect("scenario runs")
                .final_state_root()
                .expect("scenario has ticks")
                .to_hex(),
            "d5937743dc3e0dd1fec6e55f8a65094e0225df14d7e4a8dfa7e3f4076b560419"
        );
    }
}
