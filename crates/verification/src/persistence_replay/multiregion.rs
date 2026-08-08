use std::collections::BTreeSet;

use next_assets::{ContentStore, SaveStore};
use next_contracts::ids::{SchemaId, StateRoot};
use next_contracts::snapshot::world_checkpoint_with_streaming_v1_state_root;
use next_contracts::world::WorldChunkLifecycleV1;
use next_runtime::RuntimeState;
use next_world::{PreparedWorldChunkLoadV1, WorldStreamerV1};

use crate::player_fixture::prepare_fixture_project_package_with_scratch;
use crate::scratch::ScratchContext;

use super::PersistenceReplayCheckError;
use super::rpg_fixture::compatibility;

pub(super) fn verify(scratch: &ScratchContext) -> Result<(), PersistenceReplayCheckError> {
    let prepared =
        prepare_fixture_project_package_with_scratch(scratch, "nextengine.persistence-replay")
            .map_err(|error| {
                PersistenceReplayCheckError::new("prepare multiregion package", error.to_string())
            })?;
    let content_root = prepared.path().to_path_buf();
    let result = verify_package_restart(scratch, &content_root, &prepared.package);
    prepared.finish(result, |error| {
        PersistenceReplayCheckError::new("remove multiregion package", error.to_string())
    })
}

fn verify_package_restart(
    scratch: &ScratchContext,
    content_root: &std::path::Path,
    package: &next_project::ActivatedProjectPackage,
) -> Result<(), PersistenceReplayCheckError> {
    let fixture = next_reference_game::build_reference_game_session(package.project.clone())
        .map_err(|error| {
            PersistenceReplayCheckError::new("build multiregion fixture", error.to_string())
        })?;
    let route = fixture
        .world_topology()
        .ordered_multiregion_route()
        .to_vec();
    let regions = route
        .iter()
        .map(|entry| entry.region_id.clone())
        .collect::<BTreeSet<_>>();
    if route.len() != 64 || regions.len() != 4 {
        return Err(PersistenceReplayCheckError::condition(
            "multiregion route is exactly four regions and 64 chunks",
        ));
    }
    let boundary_index = (1..route.len())
        .find(|index| route[*index - 1].region_id != route[*index].region_id)
        .ok_or_else(|| {
            PersistenceReplayCheckError::condition("multiregion route crosses a region boundary")
        })?;

    let mut direct_runtime = RuntimeState::new(
        fixture.bootstrap.clone(),
        fixture.authority.clone(),
    )
    .map_err(|error| {
        PersistenceReplayCheckError::new("activate direct multiregion runtime", error.to_string())
    })?;
    let mut restart_runtime = RuntimeState::new(
        fixture.bootstrap.clone(),
        fixture.authority.clone(),
    )
    .map_err(|error| {
        PersistenceReplayCheckError::new("activate restart multiregion runtime", error.to_string())
    })?;
    let mut direct_world = WorldStreamerV1::activate(
        package.project.clone(),
        package.content_generation.clone(),
        route[0].chunk_id.clone(),
    )
    .map_err(|error| {
        PersistenceReplayCheckError::new("activate direct multiregion world", error.to_string())
    })?;
    let mut restart_world = WorldStreamerV1::activate(
        package.project.clone(),
        package.content_generation.clone(),
        route[0].chunk_id.clone(),
    )
    .map_err(|error| {
        PersistenceReplayCheckError::new("activate restart multiregion world", error.to_string())
    })?;

    for entry in &route[1..boundary_index] {
        transition(
            &mut direct_runtime,
            &mut direct_world,
            entry.chunk_id.clone(),
        )?;
        transition(
            &mut restart_runtime,
            &mut restart_world,
            entry.chunk_id.clone(),
        )?;
    }
    begin_requested(
        &mut direct_runtime,
        &mut direct_world,
        route[boundary_index].chunk_id.clone(),
    )?;
    begin_requested(
        &mut restart_runtime,
        &mut restart_world,
        route[boundary_index].chunk_id.clone(),
    )?;
    if direct_world.snapshot() != restart_world.snapshot()
        || direct_runtime.world_checkpoint().map_err(|error| {
            PersistenceReplayCheckError::new("direct requested checkpoint", error.to_string())
        })? != restart_runtime.world_checkpoint().map_err(|error| {
            PersistenceReplayCheckError::new("restart requested checkpoint", error.to_string())
        })?
    {
        return Err(PersistenceReplayCheckError::condition(
            "requested direct and restart candidates are exact",
        ));
    }

    let save_directory = scratch
        .create_directory("multiregion-save")
        .map_err(|error| {
            PersistenceReplayCheckError::new("create multiregion save directory", error.to_string())
        })?;
    let save_result = (|| {
        let mut save_compatibility = compatibility()?;
        save_compatibility.project_id =
            SchemaId::new(package.project.project_lock.project_id.as_str()).map_err(|error| {
                PersistenceReplayCheckError::new("multiregion save project ID", error.to_string())
            })?;
        save_compatibility.schema_registry_hash = package
            .project
            .schema_registry
            .schema_registry_manifest_sha256;
        save_compatibility.content_manifest_hash =
            package.project.content_manifest.content_manifest_sha256;
        save_compatibility.mechanics_lock_hash = package
            .project
            .rpg_definitions
            .mechanics_lock
            .mechanics_lock_sha256;
        let saved_checkpoint = restart_runtime.world_checkpoint().map_err(|error| {
            PersistenceReplayCheckError::new("multiregion requested checkpoint", error.to_string())
        })?;
        let saved_world = restart_world.snapshot().clone();
        let store = SaveStore::new(save_directory.path());
        store
            .commit_world_checkpoint_with_streaming(
                save_compatibility.clone(),
                &saved_checkpoint,
                &saved_world,
            )
            .map_err(|error| {
                PersistenceReplayCheckError::new(
                    "commit multiregion requested save",
                    error.to_string(),
                )
            })?;

        let direct_loaded = direct_world
            .load_pending(next_world::WORLD_CHUNK_DEFAULT_WORKERS)
            .map_err(|error| {
                PersistenceReplayCheckError::new(
                    "load uninterrupted boundary chunk",
                    error.to_string(),
                )
            })?;
        let boundary_result_hash = direct_loaded.result_hash();
        commit_loaded(&mut direct_runtime, &mut direct_world, direct_loaded)?;

        drop(restart_world);
        drop(restart_runtime);
        drop(store);

        let reopened_content = ContentStore::new(content_root);
        let reopened_package =
            next_project::activate_project_package(&reopened_content).map_err(|error| {
                PersistenceReplayCheckError::new(
                    "reactivate multiregion content",
                    error.to_string(),
                )
            })?;
        if reopened_package.project.project_lock != package.project.project_lock {
            return Err(PersistenceReplayCheckError::condition(
                "multiregion restart reactivates the exact project lock",
            ));
        }
        let reopened_fixture =
            next_reference_game::build_reference_game_session(reopened_package.project.clone())
                .map_err(|error| {
                    PersistenceReplayCheckError::new(
                        "rebuild multiregion fixture",
                        error.to_string(),
                    )
                })?;
        let reopened_save = SaveStore::new(save_directory.path());
        let loaded = reopened_save
            .load_latest(&save_compatibility)
            .map_err(|error| {
                PersistenceReplayCheckError::new(
                    "reload multiregion requested save",
                    error.to_string(),
                )
            })?;
        let loaded_world = loaded.world_streaming_snapshot.ok_or_else(|| {
            PersistenceReplayCheckError::condition("multiregion save contains the streaming owner")
        })?;
        let mut restored_runtime = RuntimeState::restore_world_checkpoint_with_definitions(
            loaded.checkpoint,
            reopened_fixture.authority,
            reopened_package.project.rpg_definitions.clone(),
        )
        .map_err(|error| {
            PersistenceReplayCheckError::new("restore multiregion runtime", error.to_string())
        })?;
        let mut restored_world = WorldStreamerV1::restore(
            reopened_package.project,
            reopened_package.content_generation,
            loaded_world,
        )
        .map_err(|error| {
            PersistenceReplayCheckError::new("restore multiregion world", error.to_string())
        })?;
        let restored_loaded = restored_world
            .load_pending(next_world::WORLD_CHUNK_DEFAULT_WORKERS)
            .map_err(|error| {
                PersistenceReplayCheckError::new(
                    "refetch restored boundary chunk",
                    error.to_string(),
                )
            })?;
        if restored_loaded.result_hash() != boundary_result_hash {
            return Err(PersistenceReplayCheckError::condition(
                "multiregion boundary result reconstructs exactly",
            ));
        }
        commit_loaded(&mut restored_runtime, &mut restored_world, restored_loaded)?;

        for entry in &route[boundary_index + 1..] {
            transition(
                &mut direct_runtime,
                &mut direct_world,
                entry.chunk_id.clone(),
            )?;
            transition(
                &mut restored_runtime,
                &mut restored_world,
                entry.chunk_id.clone(),
            )?;
        }
        let direct_root = combined_root(&direct_runtime, &direct_world)?;
        let restored_root = combined_root(&restored_runtime, &restored_world)?;
        let expected_generation = u64::try_from(route.len() - 1).map_err(|error| {
            PersistenceReplayCheckError::new("multiregion generation", error.to_string())
        })?;
        if direct_root != restored_root
            || direct_runtime.world_checkpoint().map_err(|error| {
                PersistenceReplayCheckError::new("final direct checkpoint", error.to_string())
            })? != restored_runtime.world_checkpoint().map_err(|error| {
                PersistenceReplayCheckError::new("final restored checkpoint", error.to_string())
            })?
            || direct_world.snapshot() != restored_world.snapshot()
            || direct_world.active_records() != restored_world.active_records()
            || direct_world.snapshot().generation != expected_generation
            || direct_world.snapshot().current_chunk_id
                != route.last().expect("non-empty route").chunk_id
            || direct_world
                .snapshot()
                .chunks
                .iter()
                .filter(|chunk| chunk.lifecycle == WorldChunkLifecycleV1::Active)
                .count()
                != 1
            || direct_world.snapshot().chunks.iter().any(|chunk| {
                chunk.chunk_id != route.last().expect("non-empty route").chunk_id
                    && chunk.lifecycle != WorldChunkLifecycleV1::Unloaded
            })
        {
            return Err(PersistenceReplayCheckError::condition(
                "multiregion uninterrupted and restarted roots are exact",
            ));
        }
        Ok(())
    })();
    save_directory.finish(save_result, |error| {
        PersistenceReplayCheckError::new("remove multiregion save directory", error.to_string())
    })
}

fn transition(
    runtime: &mut RuntimeState,
    world: &mut WorldStreamerV1,
    target: SchemaId,
) -> Result<(), PersistenceReplayCheckError> {
    begin_requested(runtime, world, target)?;
    let loaded = world
        .load_pending(next_world::WORLD_CHUNK_DEFAULT_WORKERS)
        .map_err(|error| {
            PersistenceReplayCheckError::new("load multiregion chunk", error.to_string())
        })?;
    commit_loaded(runtime, world, loaded)
}

fn begin_requested(
    runtime: &mut RuntimeState,
    world: &mut WorldStreamerV1,
    target: SchemaId,
) -> Result<(), PersistenceReplayCheckError> {
    let publication = world
        .prepare_begin_transition(target, runtime.next_tick())
        .map_err(|error| {
            PersistenceReplayCheckError::new("prepare multiregion request", error.to_string())
        })?;
    let prepared = runtime
        .tick_preparation()
        .prepare_with_world_streaming([], world, publication)
        .map_err(|error| {
            PersistenceReplayCheckError::new(
                "prepare multiregion requested tick",
                error.to_string(),
            )
        })?;
    let validated = runtime
        .validate_prepared_world_tick(world, prepared)
        .map_err(|error| {
            PersistenceReplayCheckError::new(
                "validate multiregion requested tick",
                error.to_string(),
            )
        })?;
    let (_, receipt) = runtime.commit_validated_world_tick(world, validated);
    if receipt.is_some() {
        return Err(PersistenceReplayCheckError::condition(
            "multiregion request has no completion receipt",
        ));
    }
    Ok(())
}

fn commit_loaded(
    runtime: &mut RuntimeState,
    world: &mut WorldStreamerV1,
    loaded: PreparedWorldChunkLoadV1,
) -> Result<(), PersistenceReplayCheckError> {
    let publication = world
        .prepare_loaded_commit(loaded, runtime.next_tick())
        .map_err(|error| {
            PersistenceReplayCheckError::new("prepare multiregion completion", error.to_string())
        })?;
    let prepared = runtime
        .tick_preparation()
        .prepare_with_world_streaming([], world, publication)
        .map_err(|error| {
            PersistenceReplayCheckError::new(
                "prepare multiregion completion tick",
                error.to_string(),
            )
        })?;
    let validated = runtime
        .validate_prepared_world_tick(world, prepared)
        .map_err(|error| {
            PersistenceReplayCheckError::new(
                "validate multiregion completion tick",
                error.to_string(),
            )
        })?;
    let (_, receipt) = runtime.commit_validated_world_tick(world, validated);
    if receipt.is_none() {
        return Err(PersistenceReplayCheckError::condition(
            "multiregion completion publishes a receipt",
        ));
    }
    Ok(())
}

fn combined_root(
    runtime: &RuntimeState,
    world: &WorldStreamerV1,
) -> Result<StateRoot, PersistenceReplayCheckError> {
    let checkpoint = runtime.world_checkpoint().map_err(|error| {
        PersistenceReplayCheckError::new("multiregion combined checkpoint", error.to_string())
    })?;
    world_checkpoint_with_streaming_v1_state_root(
        &checkpoint.runtime_snapshot,
        &checkpoint.rpg_snapshot,
        &checkpoint.physics_checkpoint,
        world.snapshot(),
    )
    .map_err(|error| {
        PersistenceReplayCheckError::new("multiregion combined root", error.to_string())
    })
}
