mod bootstrap;
mod continuation;
mod finalize;
mod replay;
mod restore;

use next_assets::SaveStore;
use next_contracts::command::WorldCommand;
use next_contracts::ids::{ContentHash, SchemaId};
use next_contracts::persistence::SaveCompatibility;
use next_contracts::snapshot::WorldCheckpointV4;
use next_contracts::world::WorldStreamingSnapshotV1;
use next_runtime::{PhysicsLaunchOptions, RuntimeState, TickReport};
use next_world::WorldStreamerV1;

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
    runtime: RuntimeState,
    initial_checkpoint: WorldCheckpointV4,
    direct_commands: Vec<WorldCommand>,
    reports: Vec<TickReport>,
}

struct RestoredScenario {
    runtime: RuntimeState,
    world: WorldStreamerV1,
    store: SaveStore,
    compatibility: SaveCompatibility,
    saved_checkpoint: WorldCheckpointV4,
    saved_world_snapshot: WorldStreamingSnapshotV1,
    _directory: CheckDirectory,
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
