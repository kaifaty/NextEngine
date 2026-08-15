mod bootstrap;
mod continuation;
mod finalize;
mod replay;
mod restore;

use next_assets::SaveStore;
use next_contracts::command::WorldCommand;
use next_contracts::ids::{ContentHash, SchemaId};
use next_contracts::persistence::{SaveCompatibility, WorldStreamingReplayInputV1};
use next_contracts::snapshot::WorldCheckpointV4;
use next_contracts::world::WorldStreamingSnapshotV1;
use next_runtime::{PhysicsLaunchOptions, RuntimeState, TickReport, WorldServicesTickCommitV1};
use next_world::{PreparedWorldStreamingPublicationV1, WorldRoutineOwnerV1, WorldStreamerV1};

use crate::NeutralPlayerFixture;
use crate::scratch::ScratchContext;

use super::{
    CheckDirectory, PersistenceReplayBackend, PersistenceReplayCheckError,
    PersistenceReplayCheckReport,
};

struct DirectScenario {
    fixture: NeutralPlayerFixture,
    content_generation: next_assets::PinnedContentGeneration,
    _project_package: crate::player_fixture::PreparedFixtureProjectPackage,
    physics_options: PhysicsLaunchOptions,
    luau_package_state_hash: ContentHash,
    wasm_plugin_state_hash: ContentHash,
    initial_chunk_id: SchemaId,
    transition_chunk_id: SchemaId,
    world: WorldStreamerV1,
    routine: WorldRoutineOwnerV1,
    runtime: RuntimeState,
    initial_checkpoint: WorldCheckpointV4,
    initial_world_snapshot: WorldStreamingSnapshotV1,
    initial_routine_snapshot_or_none: Option<next_contracts::world_routine::WorldRoutineSnapshotV1>,
    direct_commands: Vec<WorldCommand>,
    reports: Vec<TickReport>,
    world_services_commits: Vec<WorldServicesTickCommitV1>,
    replay_streaming_inputs: Vec<WorldStreamingReplayInputV1>,
    replay_direct_commands: Vec<Vec<WorldCommand>>,
}

struct RestoredScenario {
    runtime: RuntimeState,
    world: WorldStreamerV1,
    routine: WorldRoutineOwnerV1,
    store: SaveStore,
    compatibility: SaveCompatibility,
    saved_checkpoint: WorldCheckpointV4,
    saved_world_snapshot: WorldStreamingSnapshotV1,
    saved_routine_snapshot_or_none: Option<next_contracts::world_routine::WorldRoutineSnapshotV1>,
    _directory: CheckDirectory,
}

fn commit_world_services_tick(
    runtime: &mut RuntimeState,
    routine: &mut WorldRoutineOwnerV1,
    world: &mut WorldStreamerV1,
    commands: Vec<WorldCommand>,
    streaming: Option<PreparedWorldStreamingPublicationV1>,
    context: &'static str,
) -> Result<WorldServicesTickCommitV1, PersistenceReplayCheckError> {
    let prepared = match streaming {
        Some(publication) => runtime
            .tick_preparation()
            .prepare_with_world_services_and_streaming(commands, routine, world, publication),
        None => runtime
            .tick_preparation()
            .prepare_with_world_services(commands, routine, world),
    }
    .map_err(|error| PersistenceReplayCheckError::new(context, error.to_string()))?;
    let validated = runtime
        .validate_prepared_world_services_tick(routine, world, prepared)
        .map_err(|error| PersistenceReplayCheckError::new(context, error.to_string()))?;
    runtime
        .commit_validated_world_services_tick(routine, world, validated)
        .map_err(|error| PersistenceReplayCheckError::new(context, error.to_string()))
}

fn record_direct_tick(
    scenario: &mut DirectScenario,
    commands: Vec<WorldCommand>,
    streaming: Option<PreparedWorldStreamingPublicationV1>,
    streaming_input: WorldStreamingReplayInputV1,
    context: &'static str,
) -> Result<TickReport, PersistenceReplayCheckError> {
    let recorded_commands = commands.clone();
    let commit = commit_world_services_tick(
        &mut scenario.runtime,
        &mut scenario.routine,
        &mut scenario.world,
        commands,
        streaming,
        context,
    )?;
    let report = commit.runtime_report.clone();
    scenario.reports.push(report.clone());
    scenario.world_services_commits.push(commit);
    scenario.replay_streaming_inputs.push(streaming_input);
    scenario.replay_direct_commands.push(recorded_commands);
    Ok(report)
}

struct AgentEvidence {
    replay_command: WorldCommand,
    intent_id: ContentHash,
    projection_hash: ContentHash,
}

pub(crate) fn run_persistence_replay_check_for_project_with_scratch(
    scratch: &ScratchContext,
    backend: PersistenceReplayBackend,
    project_id: &str,
    physx_compatible_profile: bool,
) -> Result<PersistenceReplayCheckReport, PersistenceReplayCheckError> {
    let mut direct = bootstrap::initialize(scratch, backend, project_id, physx_compatible_profile)?;
    bootstrap::run_pre_save(&mut direct)?;
    let mut restored = restore::save_and_restore(scratch, &mut direct)?;
    let agent = continuation::run(&mut direct, &mut restored)?;
    replay::verify(&direct, &restored, &agent)?;
    finalize::complete(scratch, &direct, &restored, &agent)
}
