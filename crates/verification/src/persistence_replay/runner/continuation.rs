use next_contracts::{
    ContactPhaseV1, EventPayload, InputMappingCodeV1, PlayerActionPhaseV1, RpgEventV1, SchemaId,
    WorldCommand,
};

use crate::{player_action_sample, player_interact_sample, player_melee_sample};

use super::super::PersistenceReplayCheckError;
use super::super::replay_support::{rpg_contact_facts_from_report, transition_world};
use super::{AgentEvidence, DirectScenario, RestoredScenario};

struct PlannedAgent {
    evidence: AgentEvidence,
    direct_command: WorldCommand,
    restored_command: WorldCommand,
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
    let direct_melee = direct
        .runtime
        .run_tick([])
        .map_err(|error| PersistenceReplayCheckError::new("direct melee", error.to_string()))?;
    let restored_melee = restored
        .runtime
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
        direct.runtime.physics_snapshot().checkpoint_revision,
    );
    let restored_agent_facts = rpg_contact_facts_from_report(
        &restored_melee.contact_batch,
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
            SchemaId::new(next_contracts::CORE_MELEE_ACTION_ID)
                .expect("engine-owned melee action is valid"),
        ],
        motor_state: next_contracts::MotorCapabilityStateV1::ProceduralFallback,
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
    direct.reports.push(direct_melee);
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
    let direct_cooldown = direct
        .runtime
        .run_tick([direct_agent_command])
        .map_err(|error| {
            PersistenceReplayCheckError::new("direct cooldown retry", error.to_string())
        })?;
    let restored_cooldown = restored
        .runtime
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
    direct.reports.push(direct_cooldown);
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
    let direct_interaction = direct.runtime.run_tick([]).map_err(|error| {
        PersistenceReplayCheckError::new("direct interaction", error.to_string())
    })?;
    let restored_interaction = restored.runtime.run_tick([]).map_err(|error| {
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
    direct.reports.push(direct_interaction);
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
    let direct_left = direct
        .runtime
        .run_tick([])
        .map_err(|error| PersistenceReplayCheckError::new("direct left", error.to_string()))?;
    let restored_left = restored
        .runtime
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
    direct.reports.push(direct_left);
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
    let direct_stop = direct
        .runtime
        .run_tick([])
        .map_err(|error| PersistenceReplayCheckError::new("direct stop", error.to_string()))?;
    let restored_stop = restored
        .runtime
        .run_tick([])
        .map_err(|error| PersistenceReplayCheckError::new("restored stop", error.to_string()))?;
    if direct_stop != restored_stop
        || direct.runtime.world_checkpoint().map_err(|error| {
            PersistenceReplayCheckError::new("direct final checkpoint", error.to_string())
        })? != restored.runtime.world_checkpoint().map_err(|error| {
            PersistenceReplayCheckError::new("restored final checkpoint", error.to_string())
        })?
    {
        return Err(PersistenceReplayCheckError::condition(
            "uninterrupted and restored worlds remain exact",
        ));
    }
    direct.reports.push(direct_stop);
    Ok(())
}

fn return_world(
    direct: &mut DirectScenario,
    restored: &mut RestoredScenario,
) -> Result<(), PersistenceReplayCheckError> {
    transition_world(
        &mut direct.world,
        direct.initial_chunk_id.clone(),
        16,
        "return direct world",
    )?;
    transition_world(
        &mut restored.world,
        direct.initial_chunk_id.clone(),
        16,
        "return restored world",
    )?;
    if direct.world.snapshot() != restored.world.snapshot() {
        return Err(PersistenceReplayCheckError::condition(
            "world return remains exact after restore",
        ));
    }
    Ok(())
}
