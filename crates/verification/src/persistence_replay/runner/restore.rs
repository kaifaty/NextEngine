use next_assets::SaveStore;
use next_contracts::persistence::WorldStreamingReplayInputV1;
use next_runtime::RuntimeState;
use next_world::{
    WorldActivityOwnerV1, WorldPopulationOwnerV1, WorldRoutineOwnerV1, WorldStreamerV1,
};

use crate::scratch::ScratchContext;

use super::super::{CheckDirectory, PersistenceReplayCheckError};
use super::bootstrap::queue_saved_melee;
use super::{DirectScenario, RestoredScenario, record_direct_tick};

pub(super) fn save_and_restore(
    scratch: &ScratchContext,
    direct: &mut DirectScenario,
) -> Result<RestoredScenario, PersistenceReplayCheckError> {
    let directory = CheckDirectory::new(scratch, "save-restore")?;
    let store = SaveStore::new(directory.path());
    let compatibility = next_application::replay::replay_compatibility_for_project(
        &direct.fixture.activated_project,
        &direct.initial_checkpoint,
    )
    .map_err(|error| {
        PersistenceReplayCheckError::new("build exact replay compatibility", error.to_string())
    })?;
    let transition_tick = direct.runtime.next_tick();
    let expected_base_world_state_hash = direct.world.snapshot().state_hash().map_err(|error| {
        PersistenceReplayCheckError::new("begin world base hash", error.to_string())
    })?;
    let publication = direct
        .world
        .prepare_begin_transition(direct.transition_chunk_id.clone(), transition_tick)
        .map_err(|error| {
            PersistenceReplayCheckError::new("begin saved world transition", error.to_string())
        })?;
    let expected_next_world_state_hash = publication.next_world_state_hash();
    let transition_chunk_id = direct.transition_chunk_id.clone();
    let report = record_direct_tick(
        direct,
        Vec::new(),
        Some(publication),
        WorldStreamingReplayInputV1::BeginTransition {
            target_chunk_id: transition_chunk_id,
            expected_base_world_state_hash,
            expected_next_world_state_hash,
        },
        "joint begin saved world transition",
    )?;
    if report.tick != transition_tick
        || direct
            .world_services_commits
            .last()
            .and_then(|commit| commit.streaming_transition_or_none.as_ref())
            .is_some()
    {
        return Err(PersistenceReplayCheckError::condition(
            "requested publication has no completion receipt",
        ));
    }
    queue_saved_melee(direct)?;
    let saved_checkpoint = direct.runtime.world_checkpoint().map_err(|error| {
        PersistenceReplayCheckError::new("mid-run checkpoint", error.to_string())
    })?;
    let saved_world_snapshot = direct.world.snapshot().clone();
    let saved_routine_snapshot_or_none = direct.routine.snapshot_or_none().copied();
    let saved_routine_snapshot = saved_routine_snapshot_or_none.ok_or_else(|| {
        PersistenceReplayCheckError::condition("saved world routine owner segment exists")
    })?;
    let saved_population_snapshot =
        direct
            .population
            .snapshot_or_none()
            .cloned()
            .ok_or_else(|| {
                PersistenceReplayCheckError::condition(
                    "saved world population owner segment exists",
                )
            })?;
    let saved_activity_snapshot = direct.activity.snapshot().clone();
    let saved_agent_snapshot = direct.cognition.agent_snapshot().clone();
    let saved_memory_snapshot = direct.cognition.memory_snapshot().clone();
    let saved_physical_animation_snapshot = direct.physical_animation.snapshot().clone();
    let generation_zero = store
        .commit_world_checkpoint_with_cognition_and_physical_animation(
            compatibility.clone(),
            &saved_checkpoint,
            &saved_world_snapshot,
            Some(&saved_routine_snapshot),
            &saved_population_snapshot,
            &saved_activity_snapshot,
            &saved_agent_snapshot,
            &saved_memory_snapshot,
            &saved_physical_animation_snapshot,
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
    let restored_world = WorldStreamerV1::restore(
        direct.fixture.activated_project.clone(),
        direct.content_generation.clone(),
        loaded_world,
    )
    .map_err(|error| {
        PersistenceReplayCheckError::new("restore saved world transition", error.to_string())
    })?;
    let loaded_routine = loaded.world_routine_snapshot_or_none.ok_or_else(|| {
        PersistenceReplayCheckError::condition("loaded world routine owner segment exists")
    })?;
    let loaded_population = loaded.world_population_snapshot_or_none.ok_or_else(|| {
        PersistenceReplayCheckError::condition("loaded world population owner segment exists")
    })?;
    let loaded_activity = loaded.world_activity_snapshot_or_none.ok_or_else(|| {
        PersistenceReplayCheckError::condition("loaded world activity owner segment exists")
    })?;
    let loaded_agent = loaded.agent_cognition_snapshot_or_none.ok_or_else(|| {
        PersistenceReplayCheckError::condition("loaded agent cognition owner segment exists")
    })?;
    let loaded_memory = loaded.agent_memory_snapshot_or_none.ok_or_else(|| {
        PersistenceReplayCheckError::condition("loaded agent memory owner segment exists")
    })?;
    let loaded_physical_animation =
        loaded.physical_animation_snapshot_or_none.ok_or_else(|| {
            PersistenceReplayCheckError::condition("loaded physical animation owner segment exists")
        })?;

    let runtime = RuntimeState::restore_world_checkpoint_with_definitions_and_physics_options(
        loaded.checkpoint,
        direct.fixture.authority.clone(),
        direct.fixture.activated_project.rpg_definitions.clone(),
        direct.physics_options,
    )
    .map_err(|error| PersistenceReplayCheckError::new("restore checkpoint", error.to_string()))?;
    let routine = WorldRoutineOwnerV1::restore(
        direct
            .fixture
            .activated_project
            .world_routine_catalog_or_none,
        Some(loaded_routine),
        runtime.next_tick(),
    )
    .map_err(|error| {
        PersistenceReplayCheckError::new("restore world routine", error.to_string())
    })?;
    let population = WorldPopulationOwnerV1::restore(
        direct
            .fixture
            .activated_project
            .world_population_catalog
            .clone(),
        direct
            .fixture
            .activated_project
            .world_navigation_catalog
            .clone(),
        loaded_population,
        runtime.next_tick(),
    )
    .map_err(|error| {
        PersistenceReplayCheckError::new("restore world population", error.to_string())
    })?;
    let cognition = next_agent::cognition::StrategicAgentOwnersV1::restore(
        direct
            .fixture
            .activated_project
            .agent_cognition_catalog
            .clone(),
        loaded_agent,
        loaded_memory,
    )
    .map_err(|error| {
        PersistenceReplayCheckError::new("restore strategic cognition", error.to_string())
    })?;
    let activity = WorldActivityOwnerV1::restore(
        direct
            .fixture
            .activated_project
            .world_activity_catalog
            .clone(),
        loaded_activity,
        runtime.next_tick(),
    )
    .map_err(|error| {
        PersistenceReplayCheckError::new("restore world activity", error.to_string())
    })?;
    let physical_animation = next_reference_game::restore_reference_physical_animation_owner(
        &direct.fixture,
        loaded_physical_animation,
        runtime.physics_snapshot(),
        runtime.next_tick(),
    )
    .map_err(|error| {
        PersistenceReplayCheckError::new("restore physical animation", error.to_string())
    })?;
    runtime
        .validate_world_routine_ledger_closure(&routine)
        .map_err(|error| {
            PersistenceReplayCheckError::new("restore routine ledger closure", error.to_string())
        })?;
    runtime
        .validate_world_population_ledger_closure(&population)
        .map_err(|error| {
            PersistenceReplayCheckError::new("restore population ledger closure", error.to_string())
        })?;

    Ok(RestoredScenario {
        runtime,
        world: restored_world,
        routine,
        population,
        activity,
        cognition,
        physical_animation,
        store,
        compatibility,
        saved_checkpoint,
        saved_world_snapshot,
        saved_routine_snapshot_or_none,
        saved_population_snapshot,
        saved_activity_snapshot,
        saved_agent_snapshot,
        saved_memory_snapshot,
        saved_physical_animation_snapshot,
        _directory: directory,
    })
}
