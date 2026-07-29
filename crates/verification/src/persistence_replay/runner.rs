mod bootstrap;
mod continuation;
mod finalize;
mod replay;
mod restore;

use next_assets::SaveStore;
use next_contracts::{
    ContentHash, SaveCompatibility, SchemaId, WorldCheckpointV4, WorldCommand,
    WorldStreamingSnapshotV1,
};
use next_runtime::{PhysicsLaunchOptions, RuntimeState, TickReport};
use next_world::WorldStreamerV1;

use crate::NeutralPlayerFixture;

use super::{
    CheckDirectory, PersistenceReplayBackend, PersistenceReplayCheckError,
    PersistenceReplayCheckReport,
};

struct DirectScenario {
    fixture: NeutralPlayerFixture,
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

pub(crate) fn run_persistence_replay_check_for_project(
    backend: PersistenceReplayBackend,
    project_id: &str,
    physx_compatible_profile: bool,
) -> Result<PersistenceReplayCheckReport, PersistenceReplayCheckError> {
    let mut direct = bootstrap::initialize(backend, project_id, physx_compatible_profile)?;
    bootstrap::run_pre_save(&mut direct)?;
    let mut restored = restore::save_and_restore(&mut direct)?;
    let agent = continuation::run(&mut direct, &mut restored)?;
    replay::verify(&direct, &restored, &agent)?;
    finalize::complete(&direct, &restored, &agent)
}
