use std::fs;

use next_assets::SaveStore;
use next_contracts::command::EventPayload;
use next_contracts::ids::{PersistentId, SchemaId};
use next_contracts::mechanics::CORE_CHARACTER_HEALTH_RESOURCE_ID;
use next_contracts::physics::PhysicsPoseV1;
use next_contracts::rpg::{
    CORE_EQUIPMENT_MAIN_HAND_SLOT_ID, CORE_INTERACTIVE_OBJECT_ACTIVATED_STATE_ID,
    CORE_INTERACTIVE_OBJECT_COLLECTED_STATE_ID,
};
use next_contracts::rpg::{RpgAggregateKindV1, RpgAggregatePayloadV1, RpgSnapshotV2};
use next_contracts::snapshot::WorldCheckpointV4;

use crate::cooked_initial_interaction_outcome;
use crate::scratch::ScratchContext;

use super::super::fault_injection::{corrupt_physics_segment, corrupt_rpg_segment};
use super::super::rpg_fixture::aggregate_payload;
use super::super::{CheckDirectory, PersistenceReplayCheckError, PersistenceReplayCheckReport};
use super::{AgentEvidence, DirectScenario, RestoredScenario};

struct FinalOutcome {
    pose: PhysicsPoseV1,
    rpg_events: usize,
    interactive_object_state: SchemaId,
    dialogue_node_id: SchemaId,
    quest_state_id: SchemaId,
    npc_player_trust: i32,
    npc_health: i32,
    player_health: i32,
}

pub(super) fn complete(
    scratch: &ScratchContext,
    direct: &DirectScenario,
    restored: &RestoredScenario,
    agent: &AgentEvidence,
) -> Result<PersistenceReplayCheckReport, PersistenceReplayCheckError> {
    let final_checkpoint = direct
        .runtime
        .world_checkpoint()
        .map_err(|error| PersistenceReplayCheckError::new("final checkpoint", error.to_string()))?;
    verify_corrupt_fallbacks(scratch, direct, restored, &final_checkpoint)?;
    let outcome = read_final_outcome(direct, &final_checkpoint)?;
    let final_routine_snapshot = direct.routine.snapshot_or_none().ok_or_else(|| {
        PersistenceReplayCheckError::condition("final world routine owner segment exists")
    })?;
    let final_population_snapshot = direct.population.snapshot_or_none().ok_or_else(|| {
        PersistenceReplayCheckError::condition("final world population owner segment exists")
    })?;
    let final_state_root =
        next_contracts::snapshot::world_checkpoint_with_world_services_v1_state_root(
            &final_checkpoint.runtime_snapshot,
            &final_checkpoint.rpg_snapshot,
            &final_checkpoint.physics_checkpoint,
            direct.world.snapshot(),
            Some(final_routine_snapshot),
            Some(final_population_snapshot),
        )
        .map_err(|error| PersistenceReplayCheckError::new("final state root", error.to_string()))?;
    let final_command_ledger_hash = final_checkpoint
        .runtime_snapshot
        .command_ledger_hash()
        .map_err(|error| {
            PersistenceReplayCheckError::new("final ledger hash", error.to_string())
        })?;

    Ok(PersistenceReplayCheckReport {
        ticks: u64::try_from(direct.reports.len())
            .map_err(|error| PersistenceReplayCheckError::new("tick count", error.to_string()))?,
        generations: 2,
        final_pose: outcome.pose,
        rpg_events: u64::try_from(outcome.rpg_events).map_err(|error| {
            PersistenceReplayCheckError::new("RPG event count", error.to_string())
        })?,
        interactive_object_state: outcome.interactive_object_state,
        dialogue_node_id: outcome.dialogue_node_id,
        quest_state_id: outcome.quest_state_id,
        npc_player_trust: outcome.npc_player_trust,
        npc_health: outcome.npc_health,
        player_health: outcome.player_health,
        agent_intent_id: agent.intent_id,
        agent_projection_hash: agent.projection_hash,
        luau_package_state_hash: direct.luau_package_state_hash,
        wasm_plugin_state_hash: direct.wasm_plugin_state_hash,
        world_streaming_generation: direct.world.snapshot().generation,
        current_chunk_id: direct.world.snapshot().current_chunk_id.clone(),
        final_state_root,
        final_command_ledger_hash,
    })
}

fn verify_corrupt_fallbacks(
    scratch: &ScratchContext,
    direct: &DirectScenario,
    restored: &RestoredScenario,
    final_checkpoint: &WorldCheckpointV4,
) -> Result<(), PersistenceReplayCheckError> {
    let final_routine_snapshot = direct.routine.snapshot_or_none().ok_or_else(|| {
        PersistenceReplayCheckError::condition("final world routine owner segment exists")
    })?;
    let final_population_snapshot = direct.population.snapshot_or_none().ok_or_else(|| {
        PersistenceReplayCheckError::condition("final world population owner segment exists")
    })?;
    let generation_one = restored
        .store
        .commit_world_checkpoint_with_world_services(
            restored.compatibility.clone(),
            final_checkpoint,
            direct.world.snapshot(),
            Some(final_routine_snapshot),
            final_population_snapshot,
        )
        .map_err(|error| {
            PersistenceReplayCheckError::new("commit generation one", error.to_string())
        })?;
    if generation_one.generation != 1 {
        return Err(PersistenceReplayCheckError::condition(
            "second generation is one",
        ));
    }
    let (corrupt_path, corrupt_bytes) = corrupt_rpg_segment(
        &restored.store,
        &restored.compatibility,
        generation_one.slot,
    )?;
    let fallback = restored
        .store
        .load_latest(&restored.compatibility)
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
        || fallback.checkpoint != restored.saved_checkpoint
        || fallback.world_streaming_snapshot.as_ref() != Some(&restored.saved_world_snapshot)
        || fallback.world_routine_snapshot_or_none != restored.saved_routine_snapshot_or_none
        || fallback.world_population_snapshot_or_none
            != Some(restored.saved_population_snapshot.clone())
        || !preserved_corrupt
        || !source_unchanged
    {
        return Err(PersistenceReplayCheckError::new(
            "corrupt RPG generation falls back without rewriting bytes",
            format!(
                "generation={}, rejected={}, checkpoint_equal={}, routine_equal={}, preserved={}, source_unchanged={}",
                fallback.image.manifest.generation,
                fallback.rejected_generations.len(),
                fallback.checkpoint == restored.saved_checkpoint,
                fallback.world_routine_snapshot_or_none == restored.saved_routine_snapshot_or_none,
                preserved_corrupt,
                source_unchanged,
            ),
        ));
    }

    verify_physics_fallback(
        scratch,
        &restored.compatibility,
        &restored.saved_checkpoint,
        final_checkpoint,
    )
}

fn verify_physics_fallback(
    scratch: &ScratchContext,
    compatibility: &next_contracts::persistence::SaveCompatibility,
    saved_checkpoint: &WorldCheckpointV4,
    final_checkpoint: &WorldCheckpointV4,
) -> Result<(), PersistenceReplayCheckError> {
    let directory = CheckDirectory::new(scratch, "physics-fallback")?;
    let store = SaveStore::new(directory.path());
    store
        .commit_world_checkpoint(compatibility.clone(), saved_checkpoint)
        .map_err(|error| {
            PersistenceReplayCheckError::new("commit physics fallback baseline", error.to_string())
        })?;
    let latest = store
        .commit_world_checkpoint(compatibility.clone(), final_checkpoint)
        .map_err(|error| {
            PersistenceReplayCheckError::new("commit physics fallback candidate", error.to_string())
        })?;
    let (corrupt_path, corrupt_bytes) =
        corrupt_physics_segment(&store, compatibility, latest.slot)?;
    let fallback = store.load_latest(compatibility).map_err(|error| {
        PersistenceReplayCheckError::new("load physics fallback", error.to_string())
    })?;
    if fallback.image.manifest.generation != 0
        || fallback.checkpoint != *saved_checkpoint
        || fallback
            .rejected_generations
            .first()
            .is_none_or(|rejected| {
                !rejected
                    .original_files
                    .iter()
                    .any(|file| file.bytes == corrupt_bytes)
            })
        || fs::read(&corrupt_path).map_err(|error| {
            PersistenceReplayCheckError::new("read corrupt physics source", error.to_string())
        })? != corrupt_bytes
    {
        return Err(PersistenceReplayCheckError::condition(
            "corrupt physics generation remains a separate exact fallback regression",
        ));
    }
    Ok(())
}

fn read_final_outcome(
    direct: &DirectScenario,
    checkpoint: &WorldCheckpointV4,
) -> Result<FinalOutcome, PersistenceReplayCheckError> {
    let fixture = &direct.fixture;
    let pose = checkpoint
        .physics_checkpoint
        .snapshot
        .sorted_body_states
        .get(&fixture.physics_body_id)
        .ok_or_else(|| PersistenceReplayCheckError::condition("final capsule body exists"))?
        .pose;
    if pose.translation_micrometres != [200_000, 900_000, 200_000] {
        return Err(PersistenceReplayCheckError::condition(
            "queued movement applies exactly once",
        ));
    }
    let interactive_object_state = match aggregate_payload(
        &checkpoint.rpg_snapshot,
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
    let rpg_events = direct
        .reports
        .iter()
        .flat_map(|report| &report.events)
        .filter(|event| matches!(event.payload, EventPayload::Rpg(_)))
        .count();
    let dialogue_node_id = dialogue_node(&checkpoint.rpg_snapshot, fixture.dialogue_id)?;
    let quest_state_id = quest_state(&checkpoint.rpg_snapshot, fixture.quest_id)?;
    let (_, _, relationship_dimension_id, _) = cooked_initial_interaction_outcome(fixture);
    let expected_dialogue_node_id =
        dialogue_node(&direct.initial_checkpoint.rpg_snapshot, fixture.dialogue_id)?;
    let expected_quest_state_id =
        quest_state(&direct.initial_checkpoint.rpg_snapshot, fixture.quest_id)?;
    let expected_relationship_value = relationship_value(
        &direct.initial_checkpoint.rpg_snapshot,
        fixture,
        &relationship_dimension_id,
    )?;
    let npc_player_trust = relationship_value(
        &checkpoint.rpg_snapshot,
        fixture,
        &relationship_dimension_id,
    )?;
    let npc_health = character_health(&checkpoint.rpg_snapshot, fixture.npc_character_id)
        .ok_or_else(|| {
            PersistenceReplayCheckError::condition("final NPC health resource exists")
        })?;
    let player_health =
        character_health(&checkpoint.rpg_snapshot, fixture.body_id).ok_or_else(|| {
            PersistenceReplayCheckError::condition("final player health resource exists")
        })?;
    let pickup_is_collected = pickup_is_collected(&checkpoint.rpg_snapshot, fixture);
    let pickup_is_owned = pickup_is_owned(&checkpoint.rpg_snapshot, fixture);
    let pickup_is_equipped = pickup_is_equipped(&checkpoint.rpg_snapshot, fixture);
    if interactive_object_state.as_str() != CORE_INTERACTIVE_OBJECT_ACTIVATED_STATE_ID
        || dialogue_node_id != expected_dialogue_node_id
        || quest_state_id != expected_quest_state_id
        || npc_player_trust != expected_relationship_value
        || npc_health != 50
        || player_health != 50
        || rpg_events != 8
        || !pickup_is_collected
        || !pickup_is_owned
        || !pickup_is_equipped
    {
        return Err(PersistenceReplayCheckError::condition(
            "Rest-gated interaction preserves dialogue, quest, and relationship state",
        ));
    }

    Ok(FinalOutcome {
        pose,
        rpg_events,
        interactive_object_state,
        dialogue_node_id,
        quest_state_id,
        npc_player_trust,
        npc_health,
        player_health,
    })
}

fn dialogue_node(
    snapshot: &RpgSnapshotV2,
    dialogue_id: PersistentId,
) -> Result<SchemaId, PersistenceReplayCheckError> {
    match aggregate_payload(snapshot, RpgAggregateKindV1::Dialogue, dialogue_id) {
        Some(RpgAggregatePayloadV1::Dialogue(dialogue)) => Ok(dialogue.node_id.clone()),
        _ => Err(PersistenceReplayCheckError::condition(
            "final core dialogue exists",
        )),
    }
}

fn quest_state(
    snapshot: &RpgSnapshotV2,
    quest_id: PersistentId,
) -> Result<SchemaId, PersistenceReplayCheckError> {
    match aggregate_payload(snapshot, RpgAggregateKindV1::Quest, quest_id) {
        Some(RpgAggregatePayloadV1::Quest(quest)) => Ok(quest.state_id.clone()),
        _ => Err(PersistenceReplayCheckError::condition(
            "final core quest exists",
        )),
    }
}

fn relationship_value(
    snapshot: &RpgSnapshotV2,
    fixture: &crate::NeutralPlayerFixture,
    dimension_id: &SchemaId,
) -> Result<i32, PersistenceReplayCheckError> {
    match aggregate_payload(
        snapshot,
        RpgAggregateKindV1::Relationship,
        fixture.relationship_id,
    ) {
        Some(RpgAggregatePayloadV1::Relationship(relationship))
            if relationship.source_id == fixture.quest_giver_character_id
                && relationship.target_id == fixture.body_id =>
        {
            Ok(relationship
                .dimensions
                .iter()
                .find(|dimension| dimension.dimension_id == *dimension_id)
                .map_or(0, |dimension| dimension.value))
        }
        _ => Err(PersistenceReplayCheckError::condition(
            "final core relationship exists",
        )),
    }
}

fn character_health(snapshot: &RpgSnapshotV2, character_id: PersistentId) -> Option<i32> {
    match aggregate_payload(snapshot, RpgAggregateKindV1::Character, character_id) {
        Some(RpgAggregatePayloadV1::Character(character)) => Some(
            character
                .resources
                .iter()
                .find(|resource| resource.resource_id.as_str() == CORE_CHARACTER_HEALTH_RESOURCE_ID)
                .map_or(0, |resource| resource.current_value),
        ),
        _ => None,
    }
}

fn pickup_is_collected(snapshot: &RpgSnapshotV2, fixture: &crate::NeutralPlayerFixture) -> bool {
    matches!(
        aggregate_payload(
            snapshot,
            RpgAggregateKindV1::InteractiveObject,
            fixture.pickup_proxy_id,
        ),
        Some(RpgAggregatePayloadV1::InteractiveObject(object))
            if object.state_id.as_str() == CORE_INTERACTIVE_OBJECT_COLLECTED_STATE_ID
    )
}

fn pickup_is_owned(snapshot: &RpgSnapshotV2, fixture: &crate::NeutralPlayerFixture) -> bool {
    matches!(
        aggregate_payload(
            snapshot,
            RpgAggregateKindV1::Inventory,
            fixture.player_inventory_id,
        ),
        Some(RpgAggregatePayloadV1::Inventory(inventory))
            if inventory.item_ids == [fixture.pickup_item_id]
    )
}

fn pickup_is_equipped(snapshot: &RpgSnapshotV2, fixture: &crate::NeutralPlayerFixture) -> bool {
    matches!(
        aggregate_payload(
            snapshot,
            RpgAggregateKindV1::Equipment,
            fixture.player_equipment_id,
        ),
        Some(RpgAggregatePayloadV1::Equipment(equipment))
            if equipment.assignments.iter().any(|assignment| {
                assignment.slot_id.as_str() == CORE_EQUIPMENT_MAIN_HAND_SLOT_ID
                    && assignment.item_id == fixture.pickup_item_id
            })
    )
}
