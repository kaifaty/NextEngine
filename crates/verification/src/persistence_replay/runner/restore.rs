use next_assets::SaveStore;
use next_runtime::RuntimeState;
use next_world::WorldStreamerV1;

use crate::scratch::ScratchContext;

use super::super::rpg_fixture::compatibility;
use super::super::{CheckDirectory, PersistenceReplayCheckError};
use super::{DirectScenario, RestoredScenario};

pub(super) fn save_and_restore(
    scratch: &ScratchContext,
    direct: &mut DirectScenario,
) -> Result<RestoredScenario, PersistenceReplayCheckError> {
    let directory = CheckDirectory::new(scratch, "save-restore")?;
    let store = SaveStore::new(directory.path());
    let compatibility = compatibility()?;
    let saved_checkpoint = direct.runtime.world_checkpoint().map_err(|error| {
        PersistenceReplayCheckError::new("mid-run checkpoint", error.to_string())
    })?;
    let world_plan = direct
        .world
        .begin_transition(direct.transition_chunk_id.clone(), 11)
        .map_err(|error| {
            PersistenceReplayCheckError::new("begin saved world transition", error.to_string())
        })?;
    let mut worker_order = world_plan.ordered_required_asset_ids.clone();
    worker_order.reverse();
    let staged_world = direct
        .world
        .stage(&world_plan, &worker_order)
        .map_err(|error| {
            PersistenceReplayCheckError::new("stage saved world transition", error.to_string())
        })?;
    let saved_world_snapshot = direct.world.snapshot().clone();
    let generation_zero = store
        .commit_world_checkpoint_with_streaming(
            compatibility.clone(),
            &saved_checkpoint,
            &saved_world_snapshot,
        )
        .map_err(|error| {
            PersistenceReplayCheckError::new("commit generation zero", error.to_string())
        })?;
    if generation_zero.generation != 0 {
        return Err(PersistenceReplayCheckError::condition(
            "first generation is zero",
        ));
    }

    let loaded = store.load_latest(&compatibility).map_err(|error| {
        PersistenceReplayCheckError::new("load generation zero", error.to_string())
    })?;
    let loaded_world = loaded.world_streaming_snapshot.clone().ok_or_else(|| {
        PersistenceReplayCheckError::condition("loaded streaming owner segment exists")
    })?;
    let mut restored_world = WorldStreamerV1::restore(
        direct.fixture.activated_project.clone(),
        loaded_world,
    )
    .map_err(|error| {
        PersistenceReplayCheckError::new("restore saved world transition", error.to_string())
    })?;
    let (_, rebuilt_world) = restored_world.resume_pending().map_err(|error| {
        PersistenceReplayCheckError::new("resume saved world transition", error.to_string())
    })?;
    if rebuilt_world != staged_world {
        return Err(PersistenceReplayCheckError::condition(
            "world staging reconstructs exactly after save",
        ));
    }
    direct
        .world
        .validate_staged(&staged_world)
        .map_err(|error| {
            PersistenceReplayCheckError::new("validate direct staged world", error.to_string())
        })?;
    direct.world.commit(&staged_world, false).map_err(|error| {
        PersistenceReplayCheckError::new("commit direct world", error.to_string())
    })?;
    restored_world
        .commit(&rebuilt_world, false)
        .map_err(|error| {
            PersistenceReplayCheckError::new("commit restored world", error.to_string())
        })?;
    if direct.world.snapshot() != restored_world.snapshot() {
        return Err(PersistenceReplayCheckError::condition(
            "direct and restored world streaming states match",
        ));
    }

    let runtime = RuntimeState::restore_world_checkpoint_with_definitions_and_physics_options(
        loaded.checkpoint,
        direct.fixture.authority.clone(),
        direct.fixture.activated_project.rpg_definitions.clone(),
        direct.physics_options,
    )
    .map_err(|error| PersistenceReplayCheckError::new("restore checkpoint", error.to_string()))?;

    Ok(RestoredScenario {
        runtime,
        world: restored_world,
        store,
        compatibility,
        saved_checkpoint,
        saved_world_snapshot,
        _directory: directory,
    })
}
