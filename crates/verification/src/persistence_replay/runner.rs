use std::fs;

use next_assets::SaveStore;
use next_contracts::{
    CORE_CHARACTER_HEALTH_RESOURCE_ID, CORE_EQUIPMENT_MAIN_HAND_SLOT_ID,
    CORE_INTERACTIVE_OBJECT_ACTIVATED_STATE_ID, CORE_INTERACTIVE_OBJECT_COLLECTED_STATE_ID,
    ContactPhaseV1, EventPayload, InputMappingCodeV1, PlayerActionPhaseV1, RpgAggregateKindV1,
    RpgAggregatePayloadV1, RpgEventV1, SchemaId,
};
use next_physics_api::PhysicsBackendPolicy;
use next_runtime::{PhysicsLaunchOptions, RuntimeState};
use next_world::WorldStreamerV1;

use crate::{
    build_neutral_player_fixture, build_physx_player_fixture, cooked_interaction_outcome,
    player_action_sample, player_equip_use_sample, player_interact_sample, player_melee_sample,
    player_pickup_sample, run_replay_manifest_with_definitions_and_physics_options,
};

use super::extensions::{verify_luau_state_round_trip, verify_wasm_state_round_trip};
use super::fault_injection::{corrupt_physics_segment, corrupt_rpg_segment};
use super::replay_support::{
    compare_replay, replay_manifest, rpg_contact_facts_from_report, transition_world,
};
use super::rpg_fixture::{aggregate_payload, compatibility, initial_rpg_snapshot, rpg_commands};
use super::{
    CheckDirectory, PersistenceReplayBackend, PersistenceReplayCheckError,
    PersistenceReplayCheckReport,
};

pub(crate) fn run_persistence_replay_check_for_project(
    backend: PersistenceReplayBackend,
    project_id: &str,
    physx_compatible_profile: bool,
) -> Result<PersistenceReplayCheckReport, PersistenceReplayCheckError> {
    let physics_options = match backend {
        PersistenceReplayBackend::Reference => PhysicsLaunchOptions::default(),
        PersistenceReplayBackend::PhysX => {
            PhysicsLaunchOptions::new(PhysicsBackendPolicy::RequirePhysX)
        }
    };
    let fixture = if physx_compatible_profile {
        build_physx_player_fixture(project_id)
    } else {
        build_neutral_player_fixture(project_id)
    }
    .map_err(|error| PersistenceReplayCheckError::new("build player fixture", error.to_string()))?;
    let luau_package_state_hash = verify_luau_state_round_trip()?;
    let wasm_plugin_state_hash = verify_wasm_state_round_trip()?;
    let initial_rpg = initial_rpg_snapshot(&fixture)?;
    let initial_chunk_id = fixture
        .activated_project
        .world_partition
        .body
        .chunk_bindings
        .first()
        .ok_or_else(|| PersistenceReplayCheckError::condition("initial world chunk exists"))?
        .chunk_id
        .clone();
    let transition_chunk_id = fixture
        .activated_project
        .world_partition
        .body
        .chunk_bindings
        .get(1)
        .ok_or_else(|| PersistenceReplayCheckError::condition("second world chunk exists"))?
        .chunk_id
        .clone();
    let mut direct_world =
        WorldStreamerV1::activate(fixture.activated_project.clone(), initial_chunk_id.clone())
            .map_err(|error| {
                PersistenceReplayCheckError::new("activate world streaming", error.to_string())
            })?;
    let mut direct = RuntimeState::with_rpg_snapshot_and_physics_options(
        fixture.bootstrap.clone(),
        fixture.authority.clone(),
        initial_rpg,
        physics_options,
    )
    .map_err(|error| PersistenceReplayCheckError::new("create runtime", error.to_string()))?;

    let initial_checkpoint = direct.world_checkpoint().map_err(|error| {
        PersistenceReplayCheckError::new("initial checkpoint", error.to_string())
    })?;
    let direct_commands = rpg_commands(fixture.rpg_stream_id, fixture.principal.clone())?;
    let mut reports = Vec::new();
    for sequence in 0_u64..4 {
        let input = player_action_sample(
            &fixture,
            sequence,
            if sequence == 0 {
                PlayerActionPhaseV1::Started
            } else {
                PlayerActionPhaseV1::Performed
            },
            [0, 32_767],
            Some(
                i64::try_from(sequence).map_err(|error| {
                    PersistenceReplayCheckError::new("movement wall time", error.to_string())
                })? * 1_000,
            ),
        )
        .map_err(|error| PersistenceReplayCheckError::new("movement input", error.to_string()))?;
        direct
            .enqueue_input_sample(&fixture.principal, input)
            .map_err(|error| {
                PersistenceReplayCheckError::new("enqueue movement input", error.to_string())
            })?;
        let commands = if sequence == 0 {
            direct_commands.clone()
        } else {
            Vec::new()
        };
        reports.push(direct.run_tick(commands).map_err(|error| {
            PersistenceReplayCheckError::new("run pre-save movement", error.to_string())
        })?);
    }

    let pickup = player_pickup_sample(&fixture, 4, PlayerActionPhaseV1::Started, true, None)
        .map_err(|error| PersistenceReplayCheckError::new("pickup input", error.to_string()))?;
    direct
        .enqueue_input_sample(&fixture.principal, pickup)
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue pickup input", error.to_string())
        })?;
    reports.push(direct.run_tick([]).map_err(|error| {
        PersistenceReplayCheckError::new("run pickup interaction", error.to_string())
    })?);

    let equip = player_equip_use_sample(&fixture, 5, PlayerActionPhaseV1::Started, true, None)
        .map_err(|error| PersistenceReplayCheckError::new("equip input", error.to_string()))?;
    direct
        .enqueue_input_sample(&fixture.principal, equip)
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue equip input", error.to_string())
        })?;
    reports.push(direct.run_tick([]).map_err(|error| {
        PersistenceReplayCheckError::new("run equip interaction", error.to_string())
    })?);

    let switch_interaction =
        player_interact_sample(&fixture, 6, PlayerActionPhaseV1::Started, true, None)
            .map_err(|error| PersistenceReplayCheckError::new("switch input", error.to_string()))?;
    direct
        .enqueue_input_sample(&fixture.principal, switch_interaction)
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue switch input", error.to_string())
        })?;
    reports.push(direct.run_tick([]).map_err(|error| {
        PersistenceReplayCheckError::new("run switch interaction", error.to_string())
    })?);

    let backward = player_action_sample(
        &fixture,
        7,
        PlayerActionPhaseV1::Performed,
        [0, -32_767],
        None,
    )
    .map_err(|error| PersistenceReplayCheckError::new("backward input", error.to_string()))?;
    direct
        .enqueue_input_sample(&fixture.principal, backward)
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue backward input", error.to_string())
        })?;
    reports.push(direct.run_tick([]).map_err(|error| {
        PersistenceReplayCheckError::new("run backward movement", error.to_string())
    })?);

    for sequence in 8_u64..11 {
        let right = player_action_sample(
            &fixture,
            sequence,
            if sequence == 8 {
                PlayerActionPhaseV1::Started
            } else {
                PlayerActionPhaseV1::Performed
            },
            [32_767, 0],
            None,
        )
        .map_err(|error| PersistenceReplayCheckError::new("right input", error.to_string()))?;
        direct
            .enqueue_input_sample(&fixture.principal, right)
            .map_err(|error| {
                PersistenceReplayCheckError::new("enqueue right input", error.to_string())
            })?;
        reports.push(direct.run_tick([]).map_err(|error| {
            PersistenceReplayCheckError::new("run right movement", error.to_string())
        })?);
    }
    if !direct
        .physics_snapshot()
        .sorted_contact_continuity_states
        .values()
        .any(|contact| {
            contact.participant_low.body_id.subject_id == fixture.npc_character_id
                || contact.participant_high.body_id.subject_id == fixture.npc_character_id
        })
    {
        return Err(PersistenceReplayCheckError::condition(
            "save boundary has active NPC contact",
        ));
    }

    let queued_melee = player_melee_sample(
        &fixture,
        11,
        PlayerActionPhaseV1::Started,
        true,
        Some(11_999_999),
    )
    .map_err(|error| PersistenceReplayCheckError::new("melee input", error.to_string()))?;
    direct
        .enqueue_input_sample(&fixture.principal, queued_melee)
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue melee input", error.to_string())
        })?;

    let directory = CheckDirectory::new()?;
    let store = SaveStore::new(&directory.path);
    let compatibility = compatibility()?;
    let saved_checkpoint = direct.world_checkpoint().map_err(|error| {
        PersistenceReplayCheckError::new("mid-run checkpoint", error.to_string())
    })?;
    let world_plan = direct_world
        .begin_transition(transition_chunk_id.clone(), 11)
        .map_err(|error| {
            PersistenceReplayCheckError::new("begin saved world transition", error.to_string())
        })?;
    let mut worker_order = world_plan.ordered_required_asset_ids.clone();
    worker_order.reverse();
    let staged_world = direct_world
        .stage(&world_plan, &worker_order)
        .map_err(|error| {
            PersistenceReplayCheckError::new("stage saved world transition", error.to_string())
        })?;
    let saved_world_snapshot = direct_world.snapshot().clone();
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
        fixture.activated_project.clone(),
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
    direct_world
        .validate_staged(&staged_world)
        .map_err(|error| {
            PersistenceReplayCheckError::new("validate direct staged world", error.to_string())
        })?;
    direct_world.commit(&staged_world, false).map_err(|error| {
        PersistenceReplayCheckError::new("commit direct world", error.to_string())
    })?;
    restored_world
        .commit(&rebuilt_world, false)
        .map_err(|error| {
            PersistenceReplayCheckError::new("commit restored world", error.to_string())
        })?;
    if direct_world.snapshot() != restored_world.snapshot() {
        return Err(PersistenceReplayCheckError::condition(
            "direct and restored world streaming states match",
        ));
    }
    let mut restored = RuntimeState::restore_world_checkpoint_with_definitions_and_physics_options(
        loaded.checkpoint,
        fixture.authority.clone(),
        fixture.activated_project.rpg_definitions.clone(),
        physics_options,
    )
    .map_err(|error| PersistenceReplayCheckError::new("restore checkpoint", error.to_string()))?;

    let direct_melee = direct
        .run_tick([])
        .map_err(|error| PersistenceReplayCheckError::new("direct melee", error.to_string()))?;
    let restored_melee = restored
        .run_tick([])
        .map_err(|error| PersistenceReplayCheckError::new("restored melee", error.to_string()))?;
    if direct_melee != restored_melee
        || direct_melee.mapping_receipts.len() != 1
        || direct_melee.mapping_receipts[0].code != InputMappingCodeV1::Accepted
        || direct_melee.mapping_receipts[0]
            .derived_command_id
            .is_none()
        || direct_melee
            .contact_batch
            .events
            .iter()
            .any(|event| event.phase == ContactPhaseV1::Begin)
        || !direct_melee
            .contact_batch
            .events
            .iter()
            .any(|event| event.phase == ContactPhaseV1::Persist)
        || direct_melee
            .events
            .iter()
            .filter(|event| {
                matches!(
                    event.payload,
                    EventPayload::Rpg(RpgEventV1::CharacterResourceAdjusted { .. })
                )
            })
            .count()
            != 1
    {
        return Err(PersistenceReplayCheckError::condition(
            "queued contact-gated melee continues exactly after restore",
        ));
    }
    let direct_agent_facts = rpg_contact_facts_from_report(
        &direct_melee.contact_batch,
        direct.physics_snapshot().checkpoint_revision,
    );
    let restored_agent_facts = rpg_contact_facts_from_report(
        &restored_melee.contact_batch,
        restored.physics_snapshot().checkpoint_revision,
    );
    let direct_agent_snapshot = direct.rpg_snapshot();
    let restored_agent_snapshot = restored.rpg_snapshot();
    let direct_agent_request = next_agent::AgentPlanningRequestV1 {
        gameplay_tick: direct_melee.tick,
        world_generation: direct_world.snapshot().generation,
        decision_seed: 0x4e45_5854,
        source_character_id: fixture.npc_character_id,
        target_character_id: fixture.body_id,
        allowed_semantic_actions: vec![
            SchemaId::new(next_contracts::CORE_MELEE_ACTION_ID)
                .expect("engine-owned melee action is valid"),
        ],
        motor_state: next_contracts::MotorCapabilityStateV1::ProceduralFallback,
        ai_host_available: false,
        model_available: false,
        rpg_snapshot: &direct_agent_snapshot,
        definitions: &fixture.activated_project.rpg_definitions,
        physical_contact_facts: &direct_agent_facts,
    };
    let restored_agent_request = next_agent::AgentPlanningRequestV1 {
        rpg_snapshot: &restored_agent_snapshot,
        physical_contact_facts: &restored_agent_facts,
        ..direct_agent_request.clone()
    };
    let agent_route = next_agent::AgentCommandRouteV1 {
        issuer: fixture.agent_principal.clone(),
        stream_id: fixture.agent_stream_id,
        sequence: 0,
        target_tick: direct.next_tick(),
    };
    let direct_agent =
        next_agent::propose_world_command_v1(&direct_agent_request, agent_route.clone()).map_err(
            |error| PersistenceReplayCheckError::new("plan direct NPC action", error.to_string()),
        )?;
    let restored_agent = next_agent::propose_world_command_v1(&restored_agent_request, agent_route)
        .map_err(|error| {
            PersistenceReplayCheckError::new("plan restored NPC action", error.to_string())
        })?;
    if direct_agent.intent != restored_agent.intent
        || direct_agent.procedural_projection != restored_agent.procedural_projection
        || direct_agent.world_command != restored_agent.world_command
    {
        return Err(PersistenceReplayCheckError::condition(
            "agent plan is exact across save and chunk restore",
        ));
    }
    let replay_agent_command = direct_agent.world_command.clone();
    let direct_agent_command = direct_agent.world_command;
    let restored_agent_command = restored_agent.world_command;
    let agent_intent_id = direct_agent.intent.intent_id;
    let agent_projection_hash = direct_agent.procedural_projection.projection_hash;
    reports.push(direct_melee);

    let cooldown_retry =
        player_melee_sample(&fixture, 12, PlayerActionPhaseV1::Started, true, None).map_err(
            |error| PersistenceReplayCheckError::new("cooldown retry input", error.to_string()),
        )?;
    direct
        .enqueue_input_sample(&fixture.principal, cooldown_retry.clone())
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue direct cooldown retry", error.to_string())
        })?;
    restored
        .enqueue_input_sample(&fixture.principal, cooldown_retry)
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue restored cooldown retry", error.to_string())
        })?;
    let direct_cooldown = direct.run_tick([direct_agent_command]).map_err(|error| {
        PersistenceReplayCheckError::new("direct cooldown retry", error.to_string())
    })?;
    let restored_cooldown = restored
        .run_tick([restored_agent_command])
        .map_err(|error| {
            PersistenceReplayCheckError::new("restored cooldown retry", error.to_string())
        })?;
    if direct_cooldown != restored_cooldown
        || direct_cooldown.mapping_receipts.len() != 1
        || direct_cooldown.mapping_receipts[0].code != InputMappingCodeV1::Accepted
        || direct_cooldown.mapping_receipts[0]
            .derived_command_id
            .is_some()
        || direct_cooldown
            .events
            .iter()
            .filter(|event| {
                matches!(
                    event.payload,
                    EventPayload::Rpg(RpgEventV1::CharacterResourceAdjusted { .. })
                )
            })
            .count()
            != 1
    {
        return Err(PersistenceReplayCheckError::condition(
            "cooldown retry is a no-op while the agent command commits exactly after restore",
        ));
    }
    reports.push(direct_cooldown);

    let queued_interaction =
        player_interact_sample(&fixture, 13, PlayerActionPhaseV1::Started, true, None).map_err(
            |error| PersistenceReplayCheckError::new("interaction input", error.to_string()),
        )?;
    direct
        .enqueue_input_sample(&fixture.principal, queued_interaction.clone())
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue direct interaction", error.to_string())
        })?;
    restored
        .enqueue_input_sample(&fixture.principal, queued_interaction)
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue restored interaction", error.to_string())
        })?;
    let direct_interaction = direct.run_tick([]).map_err(|error| {
        PersistenceReplayCheckError::new("direct interaction", error.to_string())
    })?;
    let restored_interaction = restored.run_tick([]).map_err(|error| {
        PersistenceReplayCheckError::new("restored interaction", error.to_string())
    })?;
    if direct_interaction != restored_interaction
        || direct_interaction
            .events
            .iter()
            .filter(|event| {
                matches!(
                    event.payload,
                    EventPayload::Rpg(
                        RpgEventV1::DialogueAdvanced { .. }
                            | RpgEventV1::QuestTransitioned { .. }
                            | RpgEventV1::RelationshipAdjusted { .. }
                    )
                )
            })
            .count()
            != 3
    {
        return Err(PersistenceReplayCheckError::condition(
            "dialogue transaction remains exact after restored melee",
        ));
    }
    reports.push(direct_interaction);

    let left = player_action_sample(
        &fixture,
        14,
        PlayerActionPhaseV1::Performed,
        [-32_767, 0],
        None,
    )
    .map_err(|error| PersistenceReplayCheckError::new("left input", error.to_string()))?;
    direct
        .enqueue_input_sample(&fixture.principal, left.clone())
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue direct left", error.to_string())
        })?;
    restored
        .enqueue_input_sample(&fixture.principal, left)
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue restored left", error.to_string())
        })?;
    let direct_left = direct
        .run_tick([])
        .map_err(|error| PersistenceReplayCheckError::new("direct left", error.to_string()))?;
    let restored_left = restored
        .run_tick([])
        .map_err(|error| PersistenceReplayCheckError::new("restored left", error.to_string()))?;
    if direct_left != restored_left
        || !direct_left
            .contact_batch
            .events
            .iter()
            .any(|event| event.phase == ContactPhaseV1::End)
    {
        return Err(PersistenceReplayCheckError::condition(
            "left movement ends NPC contact exactly after interaction restore",
        ));
    }
    reports.push(direct_left);

    let stop = player_action_sample(&fixture, 15, PlayerActionPhaseV1::Completed, [0, 0], None)
        .map_err(|error| PersistenceReplayCheckError::new("stop input", error.to_string()))?;
    direct
        .enqueue_input_sample(&fixture.principal, stop.clone())
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue direct stop", error.to_string())
        })?;
    restored
        .enqueue_input_sample(&fixture.principal, stop)
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue restored stop", error.to_string())
        })?;
    let direct_stop = direct
        .run_tick([])
        .map_err(|error| PersistenceReplayCheckError::new("direct stop", error.to_string()))?;
    let restored_stop = restored
        .run_tick([])
        .map_err(|error| PersistenceReplayCheckError::new("restored stop", error.to_string()))?;
    if direct_stop != restored_stop
        || direct.world_checkpoint().map_err(|error| {
            PersistenceReplayCheckError::new("direct final checkpoint", error.to_string())
        })? != restored.world_checkpoint().map_err(|error| {
            PersistenceReplayCheckError::new("restored final checkpoint", error.to_string())
        })?
    {
        return Err(PersistenceReplayCheckError::condition(
            "uninterrupted and restored worlds remain exact",
        ));
    }
    reports.push(direct_stop);
    transition_world(
        &mut direct_world,
        initial_chunk_id.clone(),
        16,
        "return direct world",
    )?;
    transition_world(
        &mut restored_world,
        initial_chunk_id.clone(),
        16,
        "return restored world",
    )?;
    if direct_world.snapshot() != restored_world.snapshot() {
        return Err(PersistenceReplayCheckError::condition(
            "world return remains exact after restore",
        ));
    }

    let replay_manifest = replay_manifest(
        compatibility.clone(),
        &fixture.authority,
        initial_checkpoint,
        &reports,
        vec![
            direct_commands,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![replay_agent_command],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ],
    )?;
    let replay = run_replay_manifest_with_definitions_and_physics_options(
        &replay_manifest,
        fixture.activated_project.rpg_definitions.clone(),
        physics_options,
    )
    .map_err(|error| PersistenceReplayCheckError::new("closed-batch replay", error.to_string()))?;
    compare_replay(&direct, &reports, &replay)?;
    let mut replay_world =
        WorldStreamerV1::activate(fixture.activated_project.clone(), initial_chunk_id.clone())
            .map_err(|error| {
                PersistenceReplayCheckError::new("activate replay world", error.to_string())
            })?;
    transition_world(
        &mut replay_world,
        transition_chunk_id,
        11,
        "replay forward world",
    )?;
    transition_world(
        &mut replay_world,
        initial_chunk_id,
        16,
        "replay return world",
    )?;
    if replay_world.snapshot() != direct_world.snapshot() {
        return Err(PersistenceReplayCheckError::condition(
            "world streaming replay reaches the same state",
        ));
    }

    let final_checkpoint = direct
        .world_checkpoint()
        .map_err(|error| PersistenceReplayCheckError::new("final checkpoint", error.to_string()))?;
    let generation_one = store
        .commit_world_checkpoint_with_streaming(
            compatibility.clone(),
            &final_checkpoint,
            direct_world.snapshot(),
        )
        .map_err(|error| {
            PersistenceReplayCheckError::new("commit generation one", error.to_string())
        })?;
    if generation_one.generation != 1 {
        return Err(PersistenceReplayCheckError::condition(
            "second generation is one",
        ));
    }
    let (corrupt_path, corrupt_bytes) =
        corrupt_rpg_segment(&store, &compatibility, generation_one.slot)?;
    let fallback = store
        .load_latest(&compatibility)
        .map_err(|error| PersistenceReplayCheckError::new("load fallback", error.to_string()))?;
    let preserved_corrupt = fallback
        .rejected_generations
        .first()
        .is_some_and(|generation| {
            generation
                .original_files
                .iter()
                .any(|file| file.bytes == corrupt_bytes)
        });
    let source_unchanged = fs::read(&corrupt_path).map_err(|error| {
        PersistenceReplayCheckError::new("read corrupt source", error.to_string())
    })? == corrupt_bytes;
    if fallback.image.manifest.generation != 0
        || fallback.rejected_generations.len() != 1
        || fallback.checkpoint != saved_checkpoint
        || fallback.world_streaming_snapshot.as_ref() != Some(&saved_world_snapshot)
        || !preserved_corrupt
        || !source_unchanged
    {
        return Err(PersistenceReplayCheckError::new(
            "corrupt RPG generation falls back without rewriting bytes",
            format!(
                "generation={}, rejected={}, checkpoint_equal={}, preserved={}, source_unchanged={}",
                fallback.image.manifest.generation,
                fallback.rejected_generations.len(),
                fallback.checkpoint == saved_checkpoint,
                preserved_corrupt,
                source_unchanged,
            ),
        ));
    }

    let physics_directory = CheckDirectory::new()?;
    let physics_store = SaveStore::new(&physics_directory.path);
    physics_store
        .commit_world_checkpoint(compatibility.clone(), &saved_checkpoint)
        .map_err(|error| {
            PersistenceReplayCheckError::new("commit physics fallback baseline", error.to_string())
        })?;
    let physics_latest = physics_store
        .commit_world_checkpoint(compatibility.clone(), &final_checkpoint)
        .map_err(|error| {
            PersistenceReplayCheckError::new("commit physics fallback candidate", error.to_string())
        })?;
    let (corrupt_physics_path, corrupt_physics_bytes) =
        corrupt_physics_segment(&physics_store, &compatibility, physics_latest.slot)?;
    let physics_fallback = physics_store.load_latest(&compatibility).map_err(|error| {
        PersistenceReplayCheckError::new("load physics fallback", error.to_string())
    })?;
    if physics_fallback.image.manifest.generation != 0
        || physics_fallback.checkpoint != saved_checkpoint
        || physics_fallback
            .rejected_generations
            .first()
            .is_none_or(|rejected| {
                !rejected
                    .original_files
                    .iter()
                    .any(|file| file.bytes == corrupt_physics_bytes)
            })
        || fs::read(&corrupt_physics_path).map_err(|error| {
            PersistenceReplayCheckError::new("read corrupt physics source", error.to_string())
        })? != corrupt_physics_bytes
    {
        return Err(PersistenceReplayCheckError::condition(
            "corrupt physics generation remains a separate exact fallback regression",
        ));
    }

    let final_pose = final_checkpoint
        .physics_checkpoint
        .snapshot
        .sorted_body_states
        .get(&fixture.physics_body_id)
        .ok_or_else(|| PersistenceReplayCheckError::condition("final capsule body exists"))?
        .pose;
    if final_pose.translation_micrometres != [200_000, 900_000, 200_000] {
        return Err(PersistenceReplayCheckError::condition(
            "queued movement applies exactly once",
        ));
    }
    let interactive_object_state = match aggregate_payload(
        &final_checkpoint.rpg_snapshot,
        RpgAggregateKindV1::InteractiveObject,
        fixture.interactive_object_id,
    ) {
        Some(RpgAggregatePayloadV1::InteractiveObject(object)) => object.state_id.clone(),
        _ => {
            return Err(PersistenceReplayCheckError::condition(
                "final interactive object exists",
            ));
        }
    };
    let rpg_events = reports
        .iter()
        .flat_map(|report| &report.events)
        .filter(|event| matches!(event.payload, EventPayload::Rpg(_)))
        .count();
    let dialogue_node_id = match aggregate_payload(
        &final_checkpoint.rpg_snapshot,
        RpgAggregateKindV1::Dialogue,
        fixture.dialogue_id,
    ) {
        Some(RpgAggregatePayloadV1::Dialogue(dialogue)) => dialogue.node_id.clone(),
        _ => {
            return Err(PersistenceReplayCheckError::condition(
                "final core dialogue exists",
            ));
        }
    };
    let quest_state_id = match aggregate_payload(
        &final_checkpoint.rpg_snapshot,
        RpgAggregateKindV1::Quest,
        fixture.quest_id,
    ) {
        Some(RpgAggregatePayloadV1::Quest(quest)) => quest.state_id.clone(),
        _ => {
            return Err(PersistenceReplayCheckError::condition(
                "final core quest exists",
            ));
        }
    };
    let (
        expected_dialogue_node_id,
        expected_quest_state_id,
        relationship_dimension_id,
        expected_relationship_value,
    ) = cooked_interaction_outcome(&fixture);
    let npc_player_trust = match aggregate_payload(
        &final_checkpoint.rpg_snapshot,
        RpgAggregateKindV1::Relationship,
        fixture.relationship_id,
    ) {
        Some(RpgAggregatePayloadV1::Relationship(relationship))
            if relationship.source_id == fixture.npc_character_id
                && relationship.target_id == fixture.body_id =>
        {
            relationship
                .dimensions
                .iter()
                .find(|dimension| dimension.dimension_id == relationship_dimension_id)
                .map_or(0, |dimension| dimension.value)
        }
        _ => {
            return Err(PersistenceReplayCheckError::condition(
                "final core relationship exists",
            ));
        }
    };
    let npc_health = match aggregate_payload(
        &final_checkpoint.rpg_snapshot,
        RpgAggregateKindV1::Character,
        fixture.npc_character_id,
    ) {
        Some(RpgAggregatePayloadV1::Character(character)) => character
            .resources
            .iter()
            .find(|resource| resource.resource_id.as_str() == CORE_CHARACTER_HEALTH_RESOURCE_ID)
            .map_or(0, |resource| resource.current_value),
        _ => {
            return Err(PersistenceReplayCheckError::condition(
                "final NPC health resource exists",
            ));
        }
    };
    let player_health = match aggregate_payload(
        &final_checkpoint.rpg_snapshot,
        RpgAggregateKindV1::Character,
        fixture.body_id,
    ) {
        Some(RpgAggregatePayloadV1::Character(character)) => character
            .resources
            .iter()
            .find(|resource| resource.resource_id.as_str() == CORE_CHARACTER_HEALTH_RESOURCE_ID)
            .map_or(0, |resource| resource.current_value),
        _ => {
            return Err(PersistenceReplayCheckError::condition(
                "final player health resource exists",
            ));
        }
    };
    let pickup_is_collected = matches!(
        aggregate_payload(
            &final_checkpoint.rpg_snapshot,
            RpgAggregateKindV1::InteractiveObject,
            fixture.pickup_proxy_id,
        ),
        Some(RpgAggregatePayloadV1::InteractiveObject(object))
            if object.state_id.as_str() == CORE_INTERACTIVE_OBJECT_COLLECTED_STATE_ID
    );
    let pickup_is_owned = matches!(
        aggregate_payload(
            &final_checkpoint.rpg_snapshot,
            RpgAggregateKindV1::Inventory,
            fixture.player_inventory_id,
        ),
        Some(RpgAggregatePayloadV1::Inventory(inventory))
            if inventory.item_ids == [fixture.pickup_item_id]
    );
    let pickup_is_equipped = matches!(
        aggregate_payload(
            &final_checkpoint.rpg_snapshot,
            RpgAggregateKindV1::Equipment,
            fixture.player_equipment_id,
        ),
        Some(RpgAggregatePayloadV1::Equipment(equipment))
            if equipment.assignments.iter().any(|assignment| {
                assignment.slot_id.as_str() == CORE_EQUIPMENT_MAIN_HAND_SLOT_ID
                    && assignment.item_id == fixture.pickup_item_id
            })
    );
    if interactive_object_state.as_str() != CORE_INTERACTIVE_OBJECT_ACTIVATED_STATE_ID
        || dialogue_node_id != expected_dialogue_node_id
        || quest_state_id != expected_quest_state_id
        || npc_player_trust != expected_relationship_value
        || npc_health != 75
        || player_health != 75
        || rpg_events != 11
        || !pickup_is_collected
        || !pickup_is_owned
        || !pickup_is_equipped
    {
        return Err(PersistenceReplayCheckError::condition(
            "interaction activates object exactly once",
        ));
    }
    let final_state_root = next_contracts::world_checkpoint_with_streaming_v1_state_root(
        &final_checkpoint.runtime_snapshot,
        &final_checkpoint.rpg_snapshot,
        &final_checkpoint.physics_checkpoint,
        direct_world.snapshot(),
    )
    .map_err(|error| PersistenceReplayCheckError::new("final state root", error.to_string()))?;
    let final_command_ledger_hash = final_checkpoint
        .runtime_snapshot
        .command_ledger_hash()
        .map_err(|error| {
            PersistenceReplayCheckError::new("final ledger hash", error.to_string())
        })?;
    Ok(PersistenceReplayCheckReport {
        ticks: 16,
        generations: 2,
        final_pose,
        rpg_events: u64::try_from(rpg_events).map_err(|error| {
            PersistenceReplayCheckError::new("RPG event count", error.to_string())
        })?,
        interactive_object_state,
        dialogue_node_id,
        quest_state_id,
        npc_player_trust,
        npc_health,
        player_health,
        agent_intent_id,
        agent_projection_hash,
        luau_package_state_hash,
        wasm_plugin_state_hash,
        world_streaming_generation: direct_world.snapshot().generation,
        current_chunk_id: direct_world.snapshot().current_chunk_id.clone(),
        final_state_root,
        final_command_ledger_hash,
    })
}
