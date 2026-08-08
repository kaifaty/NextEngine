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
    let publication = direct
        .world
        .prepare_begin_transition(direct.transition_chunk_id.clone(), 11)
        .map_err(|error| {
            PersistenceReplayCheckError::new("begin saved world transition", error.to_string())
        })?;
    let validated = direct
        .world
        .validate_prepared_publication(publication, 11)
        .map_err(|error| {
            PersistenceReplayCheckError::new("validate requested world", error.to_string())
        })?;
    if direct
        .world
        .commit_validated_publication(validated)
        .is_some()
    {
        return Err(PersistenceReplayCheckError::condition(
            "requested publication has no completion receipt",
        ));
    }
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
        direct.content_generation.clone(),
        loaded_world,
    )
    .map_err(|error| {
        PersistenceReplayCheckError::new("restore saved world transition", error.to_string())
    })?;
    let direct_loaded = direct
        .world
        .load_pending(next_world::WORLD_CHUNK_DEFAULT_WORKERS)
        .map_err(|error| {
            PersistenceReplayCheckError::new("load direct packaged world", error.to_string())
        })?;
    let restored_loaded = restored_world
        .load_pending(next_world::WORLD_CHUNK_DEFAULT_WORKERS)
        .map_err(|error| {
            PersistenceReplayCheckError::new("refetch restored packaged world", error.to_string())
        })?;
    if direct_loaded.result_hash() != restored_loaded.result_hash() {
        return Err(PersistenceReplayCheckError::condition(
            "packaged result reconstructs exactly after save",
        ));
    }
    let direct_publication = direct
        .world
        .prepare_loaded_commit(direct_loaded, 12)
        .map_err(|error| {
            PersistenceReplayCheckError::new("prepare direct world completion", error.to_string())
        })?;
    let restored_publication = restored_world
        .prepare_loaded_commit(restored_loaded, 12)
        .map_err(|error| {
            PersistenceReplayCheckError::new("prepare restored world completion", error.to_string())
        })?;
    let direct_validated = direct
        .world
        .validate_prepared_publication(direct_publication, 12)
        .map_err(|error| {
            PersistenceReplayCheckError::new("validate direct completion", error.to_string())
        })?;
    let restored_validated = restored_world
        .validate_prepared_publication(restored_publication, 12)
        .map_err(|error| {
            PersistenceReplayCheckError::new("validate restored completion", error.to_string())
        })?;
    if direct
        .world
        .commit_validated_publication(direct_validated)
        .is_none()
        || restored_world
            .commit_validated_publication(restored_validated)
            .is_none()
    {
        return Err(PersistenceReplayCheckError::condition(
            "both packaged completions publish",
        ));
    }
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
