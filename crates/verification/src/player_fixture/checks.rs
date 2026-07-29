use next_contracts::{
    ActivatedProjectV1, CORE_CHARACTER_HEALTH_RESOURCE_ID, CommandLedgerHash, ContentHash,
    PhysicsPoseV1, RpgAggregateKindV1, RpgAggregatePayloadV1, SchemaId, StateRoot, domain_hash,
};
use next_physics_api::PhysicsBackendPolicy;
use next_presentation::PresentationExtractorV1;
use next_render::{ReferenceB0Renderer, RenderDevice, RenderTargetV1};
use next_runtime::PhysicsLaunchOptions;

use crate::compute_world_checkpoint_root;

use super::error::PlayCheckError;
use super::rpg::aggregate_payload;
use super::scenario::{
    GroundedCollisionScenario, run_grounded_collision_scenario,
    run_grounded_collision_scenario_with_backend,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayCheckReport {
    pub ticks: u64,
    pub final_pose: PhysicsPoseV1,
    pub events: u64,
    pub rpg_events: u64,
    pub interactive_object_state: SchemaId,
    pub dialogue_node_id: SchemaId,
    pub quest_state_id: SchemaId,
    pub npc_player_trust: i32,
    pub npc_health: i32,
    pub player_health: i32,
    pub agent_intent_id: ContentHash,
    pub agent_projection_hash: ContentHash,
    pub world_streaming_generation: u64,
    pub current_chunk_id: SchemaId,
    pub final_command_ledger_hash: CommandLedgerHash,
    pub final_state_root: StateRoot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameCheckReport {
    pub play: PlayCheckReport,
    pub presentation_snapshot_hash: ContentHash,
    pub rendered_object_count: u32,
    pub frame_plan_hash: ContentHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedGameFrameV1 {
    pub check: GameCheckReport,
    pub snapshot: next_contracts::PresentationSnapshotV2,
}

pub fn run_play_check() -> Result<PlayCheckReport, PlayCheckError> {
    let scenario = run_grounded_collision_scenario(true)?;
    play_check_report(scenario)
}

pub fn run_game_check() -> Result<GameCheckReport, PlayCheckError> {
    Ok(prepare_game_frame()?.check)
}

pub fn prepare_game_frame() -> Result<PreparedGameFrameV1, PlayCheckError> {
    let scenario = run_grounded_collision_scenario(true)?;
    prepare_game_frame_from_scenario(scenario)
}

pub fn prepare_game_frame_with_activated_project(
    activated_project: ActivatedProjectV1,
) -> Result<PreparedGameFrameV1, PlayCheckError> {
    let project_id = activated_project
        .composition_lock
        .project_id
        .as_str()
        .to_owned();
    let scenario = run_grounded_collision_scenario_with_backend(
        true,
        &project_id,
        false,
        PhysicsLaunchOptions::default(),
        Some(activated_project),
    )?;
    prepare_game_frame_from_scenario(scenario)
}

fn prepare_game_frame_from_scenario(
    scenario: GroundedCollisionScenario,
) -> Result<PreparedGameFrameV1, PlayCheckError> {
    let mut extractor = PresentationExtractorV1::new(
        scenario.project_composition_lock_hash,
        domain_hash(
            "nextengine.presentation-profile.b0.v1",
            b"sdr-reference-no-optional-features",
        ),
        8,
    )?;
    let snapshot = extractor
        .extract(
            scenario.ticks,
            scenario.project_composition_lock_hash,
            scenario.content_manifest_hash,
            scenario.runtime.physics_snapshot(),
            &scenario.presentation_bindings,
        )?
        .clone();
    let mut renderer = ReferenceB0Renderer::new();
    let frame = renderer.render(
        &snapshot,
        RenderTargetV1 {
            extent: [960, 540],
            target_revision: 1,
        },
    )?;
    let play = play_check_report(scenario)?;
    let check = GameCheckReport {
        play,
        presentation_snapshot_hash: snapshot.canonical_hash,
        rendered_object_count: frame.rendered_object_count,
        frame_plan_hash: frame.frame_plan_hash,
    };
    Ok(PreparedGameFrameV1 { check, snapshot })
}

pub fn run_play_check_with_activated_project(
    activated_project: ActivatedProjectV1,
) -> Result<PlayCheckReport, PlayCheckError> {
    let project_id = activated_project
        .composition_lock
        .project_id
        .as_str()
        .to_owned();
    let scenario = run_grounded_collision_scenario_with_backend(
        true,
        &project_id,
        false,
        PhysicsLaunchOptions::default(),
        Some(activated_project),
    )?;
    play_check_report(scenario)
}

fn play_check_report(
    scenario: GroundedCollisionScenario,
) -> Result<PlayCheckReport, PlayCheckError> {
    let checkpoint = scenario.runtime.world_checkpoint()?;
    let rpg = scenario.runtime.rpg_snapshot();
    let interactive_object_state = match aggregate_payload(
        &rpg,
        RpgAggregateKindV1::InteractiveObject,
        scenario.interactive_object_id,
    ) {
        Some(RpgAggregatePayloadV1::InteractiveObject(object)) => object.state_id.clone(),
        _ => return Err(PlayCheckError::InteractiveObjectMissing),
    };
    let dialogue_node_id =
        match aggregate_payload(&rpg, RpgAggregateKindV1::Dialogue, scenario.dialogue_id) {
            Some(RpgAggregatePayloadV1::Dialogue(dialogue)) => dialogue.node_id.clone(),
            _ => return Err(PlayCheckError::CookedDialogueMissing),
        };
    let quest_state_id = match aggregate_payload(&rpg, RpgAggregateKindV1::Quest, scenario.quest_id)
    {
        Some(RpgAggregatePayloadV1::Quest(quest)) => quest.state_id.clone(),
        _ => return Err(PlayCheckError::CookedQuestMissing),
    };
    let npc_player_trust = match aggregate_payload(
        &rpg,
        RpgAggregateKindV1::Relationship,
        scenario.relationship_id,
    ) {
        Some(RpgAggregatePayloadV1::Relationship(relationship))
            if relationship.source_id == scenario.npc_character_id
                && relationship.target_id == scenario.player_character_id =>
        {
            relationship
                .dimensions
                .iter()
                .find(|dimension| dimension.dimension_id == scenario.relationship_dimension_id)
                .map_or(0, |dimension| dimension.value)
        }
        _ => return Err(PlayCheckError::CookedNpcMissing),
    };
    let npc_health = match aggregate_payload(
        &rpg,
        RpgAggregateKindV1::Character,
        scenario.npc_character_id,
    ) {
        Some(RpgAggregatePayloadV1::Character(character)) => character
            .resources
            .iter()
            .find(|resource| resource.resource_id.as_str() == CORE_CHARACTER_HEALTH_RESOURCE_ID)
            .map_or(0, |resource| resource.current_value),
        _ => return Err(PlayCheckError::CookedNpcMissing),
    };
    let player_health = match aggregate_payload(
        &rpg,
        RpgAggregateKindV1::Character,
        scenario.player_character_id,
    ) {
        Some(RpgAggregatePayloadV1::Character(character)) => character
            .resources
            .iter()
            .find(|resource| resource.resource_id.as_str() == CORE_CHARACTER_HEALTH_RESOURCE_ID)
            .map_or(0, |resource| resource.current_value),
        _ => return Err(PlayCheckError::CookedPlayerMissing),
    };
    Ok(PlayCheckReport {
        ticks: scenario.ticks,
        final_pose: scenario.final_pose,
        events: scenario.events,
        rpg_events: scenario.rpg_events,
        interactive_object_state,
        dialogue_node_id,
        quest_state_id,
        npc_player_trust,
        npc_health,
        player_health,
        agent_intent_id: scenario
            .agent_intent_id
            .ok_or(PlayCheckError::AgentActionMissing)?,
        agent_projection_hash: scenario
            .agent_projection_hash
            .ok_or(PlayCheckError::AgentActionMissing)?,
        world_streaming_generation: scenario.world_streaming_snapshot.generation,
        current_chunk_id: scenario.world_streaming_snapshot.current_chunk_id.clone(),
        final_command_ledger_hash: checkpoint.runtime_snapshot.command_ledger_hash()?,
        final_state_root: next_contracts::world_checkpoint_with_streaming_v1_state_root(
            &checkpoint.runtime_snapshot,
            &checkpoint.rpg_snapshot,
            &checkpoint.physics_checkpoint,
            &scenario.world_streaming_snapshot,
        )?,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicsCollisionCheckReport {
    pub gameplay_ticks: u64,
    pub physics_substeps: u64,
    pub begin_contacts: u64,
    pub persist_contacts: u64,
    pub end_contacts: u64,
    pub final_pose: PhysicsPoseV1,
    pub contact_batches_hash: ContentHash,
    pub physics_checkpoint_hash: ContentHash,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PhysicsCollisionBackend {
    #[default]
    Reference,
    PhysX,
    Compare,
}

pub fn run_physics_collision_check() -> Result<PhysicsCollisionCheckReport, PlayCheckError> {
    run_physics_collision_check_with_backend(PhysicsCollisionBackend::Reference)
}

pub fn run_physics_collision_check_with_backend(
    backend: PhysicsCollisionBackend,
) -> Result<PhysicsCollisionCheckReport, PlayCheckError> {
    let scenario = match backend {
        PhysicsCollisionBackend::Reference => run_grounded_collision_scenario(false)?,
        PhysicsCollisionBackend::PhysX => run_grounded_collision_scenario_with_backend(
            false,
            "nextengine.physics-collision.physx",
            true,
            PhysicsLaunchOptions::new(PhysicsBackendPolicy::RequirePhysX),
            None,
        )?,
        PhysicsCollisionBackend::Compare => {
            let reference = run_grounded_collision_scenario_with_backend(
                false,
                "nextengine.physics-collision.compare",
                true,
                PhysicsLaunchOptions::new(PhysicsBackendPolicy::ReferenceOnly),
                None,
            )?;
            let physx = run_grounded_collision_scenario_with_backend(
                false,
                "nextengine.physics-collision.compare",
                true,
                PhysicsLaunchOptions::new(PhysicsBackendPolicy::RequirePhysX),
                None,
            )?;
            if reference.tick_reports != physx.tick_reports {
                return Err(PlayCheckError::BackendParityMismatch);
            }
            let reference_checkpoint = reference.runtime.world_checkpoint()?;
            let physx_checkpoint = physx.runtime.world_checkpoint()?;
            if reference_checkpoint != physx_checkpoint
                || compute_world_checkpoint_root(&reference_checkpoint)?
                    != compute_world_checkpoint_root(&physx_checkpoint)?
            {
                return Err(PlayCheckError::BackendParityMismatch);
            }
            let reference_replay =
                crate::persistence_replay::run_persistence_replay_check_for_project(
                    crate::PersistenceReplayBackend::Reference,
                    "nextengine.physics-collision.compare-replay",
                    true,
                )?;
            let physx_replay = crate::persistence_replay::run_persistence_replay_check_for_project(
                crate::PersistenceReplayBackend::PhysX,
                "nextengine.physics-collision.compare-replay",
                true,
            )?;
            if reference_replay != physx_replay {
                return Err(PlayCheckError::BackendParityMismatch);
            }
            physx
        }
    };
    Ok(PhysicsCollisionCheckReport {
        gameplay_ticks: scenario.ticks,
        physics_substeps: scenario.runtime.physics_snapshot().physics_tick,
        begin_contacts: scenario.begin_contacts,
        persist_contacts: scenario.persist_contacts,
        end_contacts: scenario.end_contacts,
        final_pose: scenario.final_pose,
        contact_batches_hash: scenario.contact_batches_hash,
        physics_checkpoint_hash: scenario.runtime.physics_checkpoint().checkpoint_hash()?,
    })
}
