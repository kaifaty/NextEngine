use next_contracts::command::{EventPayload, WorldCommand};
use next_contracts::ids::SchemaId;
use next_contracts::input::{InputMappingCodeV1, PlayerActionPhaseV1};
use next_contracts::persistence::WorldStreamingReplayInputV1;
use next_contracts::physics::ContactPhaseV1;
use next_contracts::rpg::RpgEventV1;

use crate::{player_action_sample, player_interact_sample, player_melee_sample};

use super::super::PersistenceReplayCheckError;
use super::super::replay_support::rpg_contact_facts_from_report;
use super::{AgentEvidence, DirectScenario, RestoredScenario, commit_world_services_tick};

struct PlannedAgent {
    evidence: AgentEvidence,
    direct_command: WorldCommand,
    restored_command: WorldCommand,
}

#[allow(clippy::too_many_arguments)]
fn run_paired_tick(
    direct: &mut DirectScenario,
    restored: &mut RestoredScenario,
    direct_commands: Vec<WorldCommand>,
    restored_commands: Vec<WorldCommand>,
    direct_streaming: Option<next_world::PreparedWorldStreamingPublicationV1>,
    restored_streaming: Option<next_world::PreparedWorldStreamingPublicationV1>,
    streaming_input: WorldStreamingReplayInputV1,
    context: &'static str,
) -> Result<next_runtime::TickReport, PersistenceReplayCheckError> {
    let replay_commands = direct_commands.clone();
    let direct_commit = commit_world_services_tick(
        &mut direct.runtime,
        &mut direct.routine,
        &mut direct.population,
        &mut direct.cognition,
        &mut direct.world,
        direct_commands,
        direct_streaming,
        context,
    )?;
    let restored_commit = commit_world_services_tick(
        &mut restored.runtime,
        &mut restored.routine,
        &mut restored.population,
        &mut restored.cognition,
        &mut restored.world,
        restored_commands,
        restored_streaming,
        context,
    )?;
    if direct_commit != restored_commit {
        return Err(PersistenceReplayCheckError::condition(context));
    }
    let report = direct_commit.runtime_report.clone();
    direct.reports.push(report.clone());
    direct.world_services_commits.push(direct_commit);
    direct.replay_streaming_inputs.push(streaming_input);
    direct.replay_direct_commands.push(replay_commands);
    Ok(report)
}

pub(super) fn run(
    direct: &mut DirectScenario,
    restored: &mut RestoredScenario,
) -> Result<AgentEvidence, PersistenceReplayCheckError> {
    let PlannedAgent {
        evidence,
        direct_command,
        restored_command,
    } = run_queued_melee_and_plan_agent(direct, restored)?;
    run_cooldown_retry(direct, restored, direct_command, restored_command)?;
    run_dialogue_transaction(direct, restored)?;
    run_left_movement(direct, restored)?;
    run_stop_and_compare(direct, restored)?;
    return_world(direct, restored)?;
    Ok(evidence)
}

fn run_queued_melee_and_plan_agent(
    direct: &mut DirectScenario,
    restored: &mut RestoredScenario,
) -> Result<PlannedAgent, PersistenceReplayCheckError> {
    let tick = direct.runtime.next_tick();
    let expected_base_world_state_hash = direct.world.snapshot().state_hash().map_err(|error| {
        PersistenceReplayCheckError::new("complete world base hash", error.to_string())
    })?;
    if restored.world.snapshot().state_hash().map_err(|error| {
        PersistenceReplayCheckError::new("restored complete world base hash", error.to_string())
    })? != expected_base_world_state_hash
    {
        return Err(PersistenceReplayCheckError::condition(
            "pending world bases match after restore",
        ));
    }
    let direct_loaded = direct
        .world
        .load_pending(next_world::WORLD_CHUNK_DEFAULT_WORKERS)
        .map_err(|error| {
            PersistenceReplayCheckError::new("load direct packaged world", error.to_string())
        })?;
    let restored_loaded = restored
        .world
        .load_pending(next_world::WORLD_CHUNK_DEFAULT_WORKERS)
        .map_err(|error| {
            PersistenceReplayCheckError::new("load restored packaged world", error.to_string())
        })?;
    let expected_loaded_result_hash = direct_loaded.result_hash();
    if restored_loaded.result_hash() != expected_loaded_result_hash {
        return Err(PersistenceReplayCheckError::condition(
            "packaged result reconstructs exactly after save",
        ));
    }
    let direct_publication = direct
        .world
        .prepare_loaded_commit(direct_loaded, tick)
        .map_err(|error| {
            PersistenceReplayCheckError::new("prepare direct world completion", error.to_string())
        })?;
    let restored_publication = restored
        .world
        .prepare_loaded_commit(restored_loaded, tick)
        .map_err(|error| {
            PersistenceReplayCheckError::new("prepare restored world completion", error.to_string())
        })?;
    let expected_next_world_state_hash = direct_publication.next_world_state_hash();
    if restored_publication.next_world_state_hash() != expected_next_world_state_hash {
        return Err(PersistenceReplayCheckError::condition(
            "completed world publications match",
        ));
    }
    let target_chunk_id = direct.transition_chunk_id.clone();
    let direct_melee = run_paired_tick(
        direct,
        restored,
        Vec::new(),
        Vec::new(),
        Some(direct_publication),
        Some(restored_publication),
        WorldStreamingReplayInputV1::CompletePendingTransition {
            target_chunk_id,
            expected_base_world_state_hash,
            expected_loaded_result_hash,
            expected_next_world_state_hash,
        },
        "queued melee and world completion",
    )?;
    if direct_melee.mapping_receipts.len() != 1
        || direct_melee.mapping_receipts[0].frame_code != InputMappingCodeV1::Accepted
        || direct_melee.mapping_receipts[0].derived_commands.is_empty()
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
        direct.runtime.physics_snapshot().checkpoint_revision,
    );
    let restored_agent_facts = rpg_contact_facts_from_report(
        &direct_melee.contact_batch,
        restored.runtime.physics_snapshot().checkpoint_revision,
    );
    let direct_agent_snapshot = direct.runtime.rpg_snapshot();
    let restored_agent_snapshot = restored.runtime.rpg_snapshot();
    let direct_agent_request = next_agent::AgentPlanningRequestV1 {
        gameplay_tick: direct_melee.tick,
        world_generation: direct.world.snapshot().generation,
        decision_seed: 0x4e45_5854,
        source_character_id: direct.fixture.npc_character_id,
        target_character_id: direct.fixture.body_id,
        allowed_semantic_actions: vec![
            SchemaId::new(next_contracts::input::CORE_MELEE_ACTION_ID)
                .expect("engine-owned melee action is valid"),
        ],
        motor_state: next_contracts::agent::MotorCapabilityStateV1::ProceduralFallback,
        ai_host_available: false,
        model_available: false,
        rpg_snapshot: &direct_agent_snapshot,
        definitions: &direct.fixture.activated_project.rpg_definitions,
        physical_contact_facts: &direct_agent_facts,
    };
    let restored_agent_request = next_agent::AgentPlanningRequestV1 {
        rpg_snapshot: &restored_agent_snapshot,
        physical_contact_facts: &restored_agent_facts,
        ..direct_agent_request.clone()
    };
    let agent_route = next_agent::AgentCommandRouteV1 {
        issuer: direct.fixture.agent_principal.clone(),
        stream_id: direct.fixture.agent_stream_id,
        sequence: 0,
        target_tick: direct.runtime.next_tick(),
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

    let evidence = AgentEvidence {
        replay_command: direct_agent.world_command.clone(),
        intent_id: direct_agent.intent.intent_id,
        projection_hash: direct_agent.procedural_projection.projection_hash,
    };
    let direct_command = direct_agent.world_command;
    let restored_command = restored_agent.world_command;
    Ok(PlannedAgent {
        evidence,
        direct_command,
        restored_command,
    })
}

fn run_cooldown_retry(
    direct: &mut DirectScenario,
    restored: &mut RestoredScenario,
    direct_agent_command: WorldCommand,
    restored_agent_command: WorldCommand,
) -> Result<(), PersistenceReplayCheckError> {
    let cooldown_retry = player_melee_sample(
        &direct.fixture,
        12,
        PlayerActionPhaseV1::Started,
        true,
        None,
    )
    .map_err(|error| PersistenceReplayCheckError::new("cooldown retry input", error.to_string()))?;
    direct
        .runtime
        .enqueue_input_sample(&direct.fixture.principal, cooldown_retry.clone())
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue direct cooldown retry", error.to_string())
        })?;
    restored
        .runtime
        .enqueue_input_sample(&direct.fixture.principal, cooldown_retry)
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue restored cooldown retry", error.to_string())
        })?;
    let direct_cooldown = run_paired_tick(
        direct,
        restored,
        vec![direct_agent_command],
        vec![restored_agent_command],
        None,
        None,
        WorldStreamingReplayInputV1::None,
        "direct/restored cooldown retry",
    )?;
    if direct_cooldown.mapping_receipts.len() != 1
        || direct_cooldown.mapping_receipts[0].frame_code != InputMappingCodeV1::Accepted
        || !direct_cooldown.mapping_receipts[0]
            .derived_commands
            .is_empty()
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
    Ok(())
}

fn run_dialogue_transaction(
    direct: &mut DirectScenario,
    restored: &mut RestoredScenario,
) -> Result<(), PersistenceReplayCheckError> {
    let queued_interaction = player_interact_sample(
        &direct.fixture,
        13,
        PlayerActionPhaseV1::Started,
        true,
        None,
    )
    .map_err(|error| PersistenceReplayCheckError::new("interaction input", error.to_string()))?;
    direct
        .runtime
        .enqueue_input_sample(&direct.fixture.principal, queued_interaction.clone())
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue direct interaction", error.to_string())
        })?;
    restored
        .runtime
        .enqueue_input_sample(&direct.fixture.principal, queued_interaction)
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue restored interaction", error.to_string())
        })?;
    let direct_interaction = run_paired_tick(
        direct,
        restored,
        Vec::new(),
        Vec::new(),
        None,
        None,
        WorldStreamingReplayInputV1::None,
        "direct/restored unavailable interaction",
    )?;
    if direct_interaction.mapping_receipts.len() != 1
        || !direct_interaction.mapping_receipts[0]
            .derived_commands
            .is_empty()
        || direct_interaction.interaction_availability.len() != 1
        || direct_interaction.interaction_availability[0].code
            != next_contracts::world_routine::InteractionAvailabilityCodeV1::WorldRoutineActivityUnavailable
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
            != 0
    {
        return Err(PersistenceReplayCheckError::condition(
            "Rest interaction remains unavailable after restored melee",
        ));
    }
    Ok(())
}

fn run_left_movement(
    direct: &mut DirectScenario,
    restored: &mut RestoredScenario,
) -> Result<(), PersistenceReplayCheckError> {
    let left = player_action_sample(
        &direct.fixture,
        14,
        PlayerActionPhaseV1::Performed,
        [-32_767, 0],
        None,
    )
    .map_err(|error| PersistenceReplayCheckError::new("left input", error.to_string()))?;
    direct
        .runtime
        .enqueue_input_sample(&direct.fixture.principal, left.clone())
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue direct left", error.to_string())
        })?;
    restored
        .runtime
        .enqueue_input_sample(&direct.fixture.principal, left)
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue restored left", error.to_string())
        })?;
    let direct_left = run_paired_tick(
        direct,
        restored,
        Vec::new(),
        Vec::new(),
        None,
        None,
        WorldStreamingReplayInputV1::None,
        "direct/restored left",
    )?;
    if !direct_left
        .contact_batch
        .events
        .iter()
        .any(|event| event.phase == ContactPhaseV1::End)
    {
        return Err(PersistenceReplayCheckError::condition(
            "left movement ends NPC contact exactly after interaction restore",
        ));
    }
    Ok(())
}

fn run_stop_and_compare(
    direct: &mut DirectScenario,
    restored: &mut RestoredScenario,
) -> Result<(), PersistenceReplayCheckError> {
    let stop = player_action_sample(
        &direct.fixture,
        15,
        PlayerActionPhaseV1::Completed,
        [0, 0],
        None,
    )
    .map_err(|error| PersistenceReplayCheckError::new("stop input", error.to_string()))?;
    direct
        .runtime
        .enqueue_input_sample(&direct.fixture.principal, stop.clone())
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue direct stop", error.to_string())
        })?;
    restored
        .runtime
        .enqueue_input_sample(&direct.fixture.principal, stop)
        .map_err(|error| {
            PersistenceReplayCheckError::new("enqueue restored stop", error.to_string())
        })?;
    let _direct_stop = run_paired_tick(
        direct,
        restored,
        Vec::new(),
        Vec::new(),
        None,
        None,
        WorldStreamingReplayInputV1::None,
        "direct/restored stop",
    )?;
    if direct.runtime.world_checkpoint().map_err(|error| {
        PersistenceReplayCheckError::new("direct final checkpoint", error.to_string())
    })? != restored.runtime.world_checkpoint().map_err(|error| {
        PersistenceReplayCheckError::new("restored final checkpoint", error.to_string())
    })? {
        return Err(PersistenceReplayCheckError::condition(
            "uninterrupted and restored worlds remain exact",
        ));
    }
    Ok(())
}

fn return_world(
    direct: &mut DirectScenario,
    restored: &mut RestoredScenario,
) -> Result<(), PersistenceReplayCheckError> {
    let begin_tick = direct.runtime.next_tick();
    let expected_base_world_state_hash = direct.world.snapshot().state_hash().map_err(|error| {
        PersistenceReplayCheckError::new("return world base hash", error.to_string())
    })?;
    let direct_begin = direct
        .world
        .prepare_begin_transition(direct.initial_chunk_id.clone(), begin_tick)
        .map_err(|error| {
            PersistenceReplayCheckError::new("prepare direct world return", error.to_string())
        })?;
    let restored_begin = restored
        .world
        .prepare_begin_transition(direct.initial_chunk_id.clone(), begin_tick)
        .map_err(|error| {
            PersistenceReplayCheckError::new("prepare restored world return", error.to_string())
        })?;
    let expected_next_world_state_hash = direct_begin.next_world_state_hash();
    if restored_begin.next_world_state_hash() != expected_next_world_state_hash {
        return Err(PersistenceReplayCheckError::condition(
            "return world begin publications match",
        ));
    }
    let initial_chunk_id = direct.initial_chunk_id.clone();
    run_paired_tick(
        direct,
        restored,
        Vec::new(),
        Vec::new(),
        Some(direct_begin),
        Some(restored_begin),
        WorldStreamingReplayInputV1::BeginTransition {
            target_chunk_id: initial_chunk_id.clone(),
            expected_base_world_state_hash,
            expected_next_world_state_hash,
        },
        "joint return world begin",
    )?;

    let complete_tick = direct.runtime.next_tick();
    let expected_base_world_state_hash = direct.world.snapshot().state_hash().map_err(|error| {
        PersistenceReplayCheckError::new("return completion base hash", error.to_string())
    })?;
    let direct_loaded = direct
        .world
        .load_pending(next_world::WORLD_CHUNK_DEFAULT_WORKERS)
        .map_err(|error| {
            PersistenceReplayCheckError::new("load direct return world", error.to_string())
        })?;
    let restored_loaded = restored
        .world
        .load_pending(next_world::WORLD_CHUNK_DEFAULT_WORKERS)
        .map_err(|error| {
            PersistenceReplayCheckError::new("load restored return world", error.to_string())
        })?;
    let expected_loaded_result_hash = direct_loaded.result_hash();
    if restored_loaded.result_hash() != expected_loaded_result_hash {
        return Err(PersistenceReplayCheckError::condition(
            "return world loaded results match",
        ));
    }
    let direct_complete = direct
        .world
        .prepare_loaded_commit(direct_loaded, complete_tick)
        .map_err(|error| {
            PersistenceReplayCheckError::new("prepare direct return completion", error.to_string())
        })?;
    let restored_complete = restored
        .world
        .prepare_loaded_commit(restored_loaded, complete_tick)
        .map_err(|error| {
            PersistenceReplayCheckError::new(
                "prepare restored return completion",
                error.to_string(),
            )
        })?;
    let expected_next_world_state_hash = direct_complete.next_world_state_hash();
    if restored_complete.next_world_state_hash() != expected_next_world_state_hash {
        return Err(PersistenceReplayCheckError::condition(
            "return world completion publications match",
        ));
    }
    run_paired_tick(
        direct,
        restored,
        Vec::new(),
        Vec::new(),
        Some(direct_complete),
        Some(restored_complete),
        WorldStreamingReplayInputV1::CompletePendingTransition {
            target_chunk_id: initial_chunk_id,
            expected_base_world_state_hash,
            expected_loaded_result_hash,
            expected_next_world_state_hash,
        },
        "joint return world completion",
    )?;
    if direct.world.snapshot() != restored.world.snapshot() {
        return Err(PersistenceReplayCheckError::condition(
            "world return remains exact after restore",
        ));
    }
    Ok(())
}
