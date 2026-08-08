use next_contracts::command::EventPayload;
use next_contracts::ids::{ContentHash, PersistentId, SchemaId, StateRoot};
use next_contracts::input::{CORE_MELEE_ACTION_ID, PlayerActionPhaseV1};
use next_contracts::mechanics::CORE_CHARACTER_HEALTH_RESOURCE_ID;
use next_contracts::physics::{ContactPhaseV1, PhysicsBodyIdV1, PhysicsPoseV1};
use next_contracts::project::ActivatedProjectV2;
use next_contracts::rpg::{
    RpgAggregateKindV1, RpgAggregatePayloadV1, RpgPhysicalContactFactV1, RpgSnapshotV2,
};
use next_presentation::PresentationBindingV1;
use next_runtime::{PhysicsLaunchOptions, RuntimeState};
use next_world::WorldStreamerV1;

use crate::ReferenceGameError;
use crate::input::NormalizedReferenceInputV1;
use crate::rpg::{aggregate_payload, cooked_interaction_outcome, cooked_project_rpg_snapshot};
use crate::session::{
    ReferenceGameSession, build_reference_game_session, build_reference_game_session_with_profile,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceStageCheckpointV1 {
    pub state_root: StateRoot,
    pub quest_state_id: SchemaId,
    pub dialogue_node_id: SchemaId,
    pub player_inventory_contains_pickup: bool,
    pub player_equipment_contains_pickup: bool,
    pub npc_health: i32,
    pub player_health: i32,
    pub relay_state_id: SchemaId,
    pub current_chunk_id: SchemaId,
}

pub struct ReferenceRunOutcomeV1 {
    pub runtime: RuntimeState,
    pub ticks: u64,
    pub final_pose: PhysicsPoseV1,
    pub events: u64,
    pub rpg_events: u64,
    pub begin_contacts: u64,
    pub persist_contacts: u64,
    pub end_contacts: u64,
    pub contact_batches_hash: ContentHash,
    pub interactive_object_id: PersistentId,
    pub npc_character_id: PersistentId,
    pub quest_giver_character_id: PersistentId,
    pub player_character_id: PersistentId,
    pub dialogue_id: PersistentId,
    pub quest_id: PersistentId,
    pub pickup_item_id: PersistentId,
    pub npc_weapon_item_id: PersistentId,
    pub item_display_text_id: SchemaId,
    pub quest_display_text_id: SchemaId,
    pub relationship_id: PersistentId,
    pub relationship_dimension_id: SchemaId,
    pub project_composition_lock_hash: ContentHash,
    pub content_manifest_hash: ContentHash,
    pub render_content_catalog: next_contracts::render_content::RenderContentCatalogV1,
    pub presentation_bindings: Vec<PresentationBindingV1>,
    pub tick_reports: Vec<next_runtime::TickReport>,
    pub world_streaming_snapshot: next_contracts::world::WorldStreamingSnapshotV1,
    pub agent_intent_id: Option<ContentHash>,
    pub agent_projection_hash: Option<ContentHash>,
    pub stage_checkpoint_roots: Vec<StateRoot>,
    pub stage_checkpoints: Vec<ReferenceStageCheckpointV1>,
}

enum ScenarioAction {
    Movement(PlayerActionPhaseV1, [i16; 2]),
    Interaction,
    Pickup,
    EquipUse,
    Melee,
    AgentMelee,
    ChunkTransition(SchemaId, bool),
    Checkpoint,
}

pub fn run_reference_game(
    activated_project: ActivatedProjectV2,
    include_interaction: bool,
) -> Result<ReferenceRunOutcomeV1, ReferenceGameError> {
    run_reference_game_with_backend(
        include_interaction,
        false,
        PhysicsLaunchOptions::default(),
        activated_project,
    )
}

pub fn run_reference_game_with_backend(
    include_interaction: bool,
    physx_compatible: bool,
    physics_options: PhysicsLaunchOptions,
    activated_project: ActivatedProjectV2,
) -> Result<ReferenceRunOutcomeV1, ReferenceGameError> {
    let fixture = if physx_compatible {
        build_reference_game_session_with_profile(activated_project, true)?
    } else {
        build_reference_game_session(activated_project)?
    };
    let (_, _, relationship_dimension_id, _) = cooked_interaction_outcome(&fixture);
    let item_display_text_id = crate::ui::reference_item_display_text_id(&fixture)?;
    let quest_display_text_id = crate::ui::reference_quest_display_text_id(&fixture)?;
    let rpg_snapshot = if include_interaction {
        cooked_project_rpg_snapshot(&fixture)
    } else {
        RpgSnapshotV2::default()
    };
    let mut runtime = RuntimeState::with_rpg_snapshot_and_physics_options(
        fixture.bootstrap.clone(),
        fixture.authority.clone(),
        rpg_snapshot,
        physics_options,
    )?;
    let mut input_producer = NormalizedReferenceInputV1::new(&fixture)?;
    let initial_chunk_id = fixture
        .activated_project
        .world_partition
        .body
        .chunk_bindings
        .first()
        .ok_or(ReferenceGameError::WorldPartitionEmpty)?
        .chunk_id
        .clone();
    let transition_chunk_id = fixture
        .activated_project
        .world_partition
        .body
        .chunk_bindings
        .get(1)
        .ok_or(ReferenceGameError::WorldPartitionEmpty)?
        .chunk_id
        .clone();
    let mut world_streamer =
        WorldStreamerV1::activate(fixture.activated_project.clone(), initial_chunk_id.clone())?;
    let mut inputs = Vec::new();
    if include_interaction {
        inputs.extend([
            ScenarioAction::Interaction,
            ScenarioAction::Checkpoint,
            ScenarioAction::Movement(PlayerActionPhaseV1::Started, [0, 32_767]),
            ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [0, 32_767]),
            ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [0, 32_767]),
            ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [0, 32_767]),
            ScenarioAction::Pickup,
            ScenarioAction::EquipUse,
            ScenarioAction::ChunkTransition(transition_chunk_id, true),
            ScenarioAction::Movement(PlayerActionPhaseV1::Started, [32_767, 0]),
            ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [32_767, 0]),
            ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [32_767, 0]),
            ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [32_767, 0]),
            ScenarioAction::AgentMelee,
            ScenarioAction::Melee,
            ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [0, 32_767]),
            ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [0, 32_767]),
            ScenarioAction::Melee,
            ScenarioAction::Checkpoint,
            ScenarioAction::Interaction,
            ScenarioAction::Checkpoint,
            ScenarioAction::ChunkTransition(initial_chunk_id, false),
            ScenarioAction::Movement(PlayerActionPhaseV1::Started, [-32_767, 0]),
            ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [-32_767, 0]),
            ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [-32_767, 0]),
            ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [-32_767, 0]),
            ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [-32_767, 0]),
            ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [-32_767, 0]),
            ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [-32_767, 0]),
            ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [-32_767, 0]),
            ScenarioAction::Movement(PlayerActionPhaseV1::Started, [0, -32_767]),
            ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [0, -32_767]),
            ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [0, -32_767]),
            ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [0, -32_767]),
            ScenarioAction::Interaction,
            ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [-32_767, 0]),
            ScenarioAction::Movement(PlayerActionPhaseV1::Completed, [0, 0]),
        ]);
    } else {
        inputs.extend([
            ScenarioAction::Movement(PlayerActionPhaseV1::Started, [0, 32_767]),
            ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [0, 32_767]),
            ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [0, 32_767]),
            ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [0, 32_767]),
            ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [0, -32_767]),
            ScenarioAction::Movement(PlayerActionPhaseV1::Completed, [0, 0]),
        ]);
    }
    let mut events = 0_u64;
    let mut rpg_events = 0_u64;
    let mut begin_contacts = 0_u64;
    let mut persist_contacts = 0_u64;
    let mut end_contacts = 0_u64;
    let mut contact_preimage = b"nextengine.physics-collision-check.contacts.v1\0".to_vec();
    let mut tick_reports: Vec<next_runtime::TickReport> = Vec::new();
    let mut sequence = 0_u64;
    let mut agent_intent_id = None;
    let mut agent_projection_hash = None;
    let mut stage_checkpoint_roots = Vec::new();
    let mut stage_checkpoints = Vec::new();
    for action in inputs {
        if matches!(&action, ScenarioAction::Checkpoint) {
            let checkpoint = runtime.world_checkpoint()?;
            let expected_root = checkpoint.state_root;
            runtime = RuntimeState::restore_world_checkpoint_with_definitions_and_physics_options(
                checkpoint,
                fixture.authority.clone(),
                fixture.activated_project.rpg_definitions.clone(),
                physics_options,
            )?;
            let restored_root = runtime.world_checkpoint()?.state_root;
            if restored_root != expected_root {
                return Err(ReferenceGameError::RecoveryInvalid);
            }
            stage_checkpoint_roots.push(restored_root);
            stage_checkpoints.push(reference_stage_checkpoint(
                &runtime,
                &fixture,
                restored_root,
                world_streamer.snapshot().current_chunk_id.clone(),
            )?);
            continue;
        }
        if let ScenarioAction::ChunkTransition(target_chunk_id, save_restore) = action {
            let rpg_before = runtime.rpg_snapshot();
            let plan = world_streamer.begin_transition(target_chunk_id, sequence)?;
            let mut worker_order = plan.ordered_required_asset_ids.clone();
            worker_order.reverse();
            let staged = world_streamer.stage(&plan, &worker_order)?;
            if save_restore {
                let saved = world_streamer.snapshot().canonical_bytes()?;
                let decoded =
                    next_contracts::world::WorldStreamingSnapshotV1::from_canonical_bytes(
                        &saved,
                        next_contracts::canonical::CanonicalDecodeLimits::default(),
                    )?;
                world_streamer =
                    WorldStreamerV1::restore(fixture.activated_project.clone(), decoded)?;
                let (_, rebuilt) = world_streamer.resume_pending()?;
                if rebuilt != staged {
                    return Err(ReferenceGameError::WorldStreamingResumeMismatch);
                }
            } else {
                world_streamer.validate_staged(&staged)?;
            }
            world_streamer.commit(&staged, false)?;
            if runtime.rpg_snapshot() != rpg_before {
                return Err(ReferenceGameError::WorldStreamingMutatedRpg);
            }
            continue;
        }
        if matches!(&action, ScenarioAction::AgentMelee) {
            let previous = tick_reports
                .last()
                .ok_or(ReferenceGameError::AgentActionMissing)?;
            let facts = rpg_contact_facts_from_report(
                &previous.contact_batch,
                runtime.physics_snapshot().checkpoint_revision,
            );
            let agent_rpg_snapshot = runtime.rpg_snapshot();
            let request = next_agent::AgentPlanningRequestV1 {
                gameplay_tick: previous.tick,
                world_generation: world_streamer.snapshot().generation,
                decision_seed: 0x4e45_5854,
                source_character_id: fixture.npc_character_id,
                target_character_id: fixture.body_id,
                allowed_semantic_actions: vec![
                    SchemaId::new(CORE_MELEE_ACTION_ID)
                        .expect("engine-owned melee action is valid"),
                ],
                motor_state: next_contracts::agent::MotorCapabilityStateV1::ProceduralFallback,
                ai_host_available: false,
                model_available: false,
                rpg_snapshot: &agent_rpg_snapshot,
                definitions: &fixture.activated_project.rpg_definitions,
                physical_contact_facts: &facts,
            };
            let planned = next_agent::propose_world_command_v1(
                &request,
                next_agent::AgentCommandRouteV1 {
                    issuer: fixture.agent_principal.clone(),
                    stream_id: fixture.agent_stream_id,
                    sequence: 0,
                    target_tick: runtime.next_tick(),
                },
            )?;
            let report = runtime.run_tick([planned.world_command])?;
            if !report.results.iter().any(|result| {
                matches!(
                    result.disposition,
                    next_runtime::CommandDisposition::Committed
                )
            }) {
                return Err(ReferenceGameError::AgentCommandRejected);
            }
            agent_intent_id = Some(planned.intent.intent_id);
            agent_projection_hash = Some(planned.procedural_projection.projection_hash);
            accumulate_report(
                &report,
                &mut events,
                &mut rpg_events,
                &mut begin_contacts,
                &mut persist_contacts,
                &mut end_contacts,
                &mut contact_preimage,
            )?;
            tick_reports.push(report);
            continue;
        }
        let sample = match action {
            ScenarioAction::Movement(_phase, direction) => {
                input_producer.movement_sample(sequence, direction)?
            }
            ScenarioAction::Interaction => input_producer
                .semantic_sample(sequence, next_contracts::input::CORE_INTERACT_ACTION_ID)?,
            ScenarioAction::Pickup => input_producer
                .semantic_sample(sequence, next_contracts::input::CORE_PICKUP_ACTION_ID)?,
            ScenarioAction::EquipUse => input_producer
                .semantic_sample(sequence, next_contracts::input::CORE_EQUIP_USE_ACTION_ID)?,
            ScenarioAction::Melee => {
                input_producer.semantic_sample(sequence, CORE_MELEE_ACTION_ID)?
            }
            ScenarioAction::AgentMelee => unreachable!("handled before input mapping"),
            ScenarioAction::ChunkTransition(_, _) => unreachable!("handled before input mapping"),
            ScenarioAction::Checkpoint => unreachable!("handled before input mapping"),
        };
        runtime.enqueue_input_sample(&fixture.principal, sample)?;
        let report = runtime.run_tick([])?;
        accumulate_report(
            &report,
            &mut events,
            &mut rpg_events,
            &mut begin_contacts,
            &mut persist_contacts,
            &mut end_contacts,
            &mut contact_preimage,
        )?;
        tick_reports.push(report);
        sequence = sequence
            .checked_add(1)
            .ok_or(ReferenceGameError::CountOverflow)?;
    }
    let final_pose = runtime
        .physics_snapshot()
        .sorted_body_states
        .get(&fixture.physics_body_id)
        .ok_or(ReferenceGameError::BodyMissing)?
        .pose;
    let ticks = runtime.next_tick();
    let world_streaming_snapshot = world_streamer.snapshot();
    let presentation_bindings = fixture_presentation_bindings(&fixture, &runtime.rpg_snapshot())?;
    Ok(ReferenceRunOutcomeV1 {
        ticks,
        final_pose,
        events,
        rpg_events,
        runtime,
        begin_contacts,
        persist_contacts,
        end_contacts,
        contact_batches_hash: ContentHash::from_bytes(next_contracts::canonical::sha256(
            &contact_preimage,
        )),
        interactive_object_id: fixture.interactive_object_id,
        npc_character_id: fixture.npc_character_id,
        quest_giver_character_id: fixture.quest_giver_character_id,
        player_character_id: fixture.body_id,
        dialogue_id: fixture.dialogue_id,
        quest_id: fixture.quest_id,
        pickup_item_id: fixture.pickup_item_id,
        npc_weapon_item_id: fixture.npc_weapon_item_id,
        item_display_text_id,
        quest_display_text_id,
        relationship_id: fixture.relationship_id,
        relationship_dimension_id,
        project_composition_lock_hash: fixture
            .activated_project
            .composition_lock
            .composition_lock_sha256,
        content_manifest_hash: fixture
            .activated_project
            .content_manifest
            .content_manifest_sha256,
        render_content_catalog: fixture.activated_project.render_content_catalog.clone(),
        presentation_bindings,
        tick_reports,
        world_streaming_snapshot: world_streaming_snapshot.clone(),
        agent_intent_id,
        agent_projection_hash,
        stage_checkpoint_roots,
        stage_checkpoints,
    })
}

fn reference_stage_checkpoint(
    runtime: &RuntimeState,
    fixture: &ReferenceGameSession,
    state_root: StateRoot,
    current_chunk_id: SchemaId,
) -> Result<ReferenceStageCheckpointV1, ReferenceGameError> {
    let rpg = runtime.rpg_snapshot();
    let quest_state_id = match aggregate_payload(&rpg, RpgAggregateKindV1::Quest, fixture.quest_id)
    {
        Some(RpgAggregatePayloadV1::Quest(quest)) => quest.state_id.clone(),
        _ => return Err(ReferenceGameError::RecoveryInvalid),
    };
    let dialogue_node_id =
        match aggregate_payload(&rpg, RpgAggregateKindV1::Dialogue, fixture.dialogue_id) {
            Some(RpgAggregatePayloadV1::Dialogue(dialogue)) => dialogue.node_id.clone(),
            _ => return Err(ReferenceGameError::RecoveryInvalid),
        };
    let player_inventory_contains_pickup = match aggregate_payload(
        &rpg,
        RpgAggregateKindV1::Inventory,
        fixture.player_inventory_id,
    ) {
        Some(RpgAggregatePayloadV1::Inventory(inventory)) => {
            inventory.item_ids.contains(&fixture.pickup_item_id)
        }
        _ => return Err(ReferenceGameError::RecoveryInvalid),
    };
    let player_equipment_contains_pickup = match aggregate_payload(
        &rpg,
        RpgAggregateKindV1::Equipment,
        fixture.player_equipment_id,
    ) {
        Some(RpgAggregatePayloadV1::Equipment(equipment)) => equipment
            .assignments
            .iter()
            .any(|assignment| assignment.item_id == fixture.pickup_item_id),
        _ => return Err(ReferenceGameError::RecoveryInvalid),
    };
    let health =
        |character_id| match aggregate_payload(&rpg, RpgAggregateKindV1::Character, character_id) {
            Some(RpgAggregatePayloadV1::Character(character)) => character
                .resources
                .iter()
                .find(|resource| resource.resource_id.as_str() == CORE_CHARACTER_HEALTH_RESOURCE_ID)
                .map(|resource| resource.current_value),
            _ => None,
        };
    let relay_state_id = match aggregate_payload(
        &rpg,
        RpgAggregateKindV1::InteractiveObject,
        fixture.interactive_object_id,
    ) {
        Some(RpgAggregatePayloadV1::InteractiveObject(relay)) => relay.state_id.clone(),
        _ => return Err(ReferenceGameError::RecoveryInvalid),
    };
    Ok(ReferenceStageCheckpointV1 {
        state_root,
        quest_state_id,
        dialogue_node_id,
        player_inventory_contains_pickup,
        player_equipment_contains_pickup,
        npc_health: health(fixture.npc_character_id).ok_or(ReferenceGameError::RecoveryInvalid)?,
        player_health: health(fixture.body_id).ok_or(ReferenceGameError::RecoveryInvalid)?,
        relay_state_id,
        current_chunk_id,
    })
}

#[allow(
    clippy::too_many_arguments,
    reason = "the acceptance accumulator updates the complete deterministic report"
)]
fn accumulate_report(
    report: &next_runtime::TickReport,
    events: &mut u64,
    rpg_events: &mut u64,
    begin_contacts: &mut u64,
    persist_contacts: &mut u64,
    end_contacts: &mut u64,
    contact_preimage: &mut Vec<u8>,
) -> Result<(), ReferenceGameError> {
    *events = events
        .checked_add(
            u64::try_from(report.events.len()).map_err(|_| ReferenceGameError::CountOverflow)?,
        )
        .ok_or(ReferenceGameError::CountOverflow)?;
    *rpg_events = rpg_events
        .checked_add(
            u64::try_from(
                report
                    .events
                    .iter()
                    .filter(|event| matches!(&event.payload, EventPayload::Rpg(_)))
                    .count(),
            )
            .map_err(|_| ReferenceGameError::CountOverflow)?,
        )
        .ok_or(ReferenceGameError::CountOverflow)?;
    for contact in &report.contact_batch.events {
        let count = match contact.phase {
            ContactPhaseV1::Begin => &mut *begin_contacts,
            ContactPhaseV1::Persist => &mut *persist_contacts,
            ContactPhaseV1::End => &mut *end_contacts,
        };
        *count = count
            .checked_add(1)
            .ok_or(ReferenceGameError::CountOverflow)?;
    }
    contact_preimage.extend_from_slice(report.contact_batch.batch_hash.as_bytes());
    Ok(())
}

fn rpg_contact_facts_from_report(
    batch: &next_contracts::physics::ClosedPhysicsContactBatchV1,
    physics_checkpoint_revision: u64,
) -> Vec<RpgPhysicalContactFactV1> {
    let mut facts = batch
        .events
        .iter()
        .filter(|event| matches!(event.phase, ContactPhaseV1::Begin | ContactPhaseV1::Persist))
        .filter_map(|event| {
            let first = event.participant_low.body_id.subject_id;
            let second = event.participant_high.body_id.subject_id;
            if first == second {
                return None;
            }
            let (subject_low, subject_high) = if first < second {
                (first, second)
            } else {
                (second, first)
            };
            Some(RpgPhysicalContactFactV1 {
                gameplay_tick: batch.gameplay_tick,
                contact_id: event.contact_id,
                subject_low,
                subject_high,
                physics_checkpoint_revision,
                source_snapshot_hash: event.source_snapshot_hash,
                contact_batch_hash: batch.batch_hash,
            })
        })
        .collect::<Vec<_>>();
    facts.sort_unstable();
    facts.dedup();
    facts
}

pub(super) fn fixture_presentation_bindings(
    fixture: &ReferenceGameSession,
    rpg: &RpgSnapshotV2,
) -> Result<Vec<PresentationBindingV1>, ReferenceGameError> {
    let revision = |asset_id| {
        fixture
            .activated_project
            .content_manifest
            .body
            .asset_entries
            .iter()
            .find(|entry| entry.asset_revision.asset_id == asset_id)
            .map(|entry| entry.asset_revision)
            .ok_or(ReferenceGameError::PresentationAssetMissing)
    };
    let mesh = |asset_id| {
        let asset_revision = revision(asset_id)?;
        let bounds = fixture
            .activated_project
            .render_content_catalog
            .meshes()
            .iter()
            .find(|mesh| mesh.asset_id() == asset_revision.asset_id)
            .map(next_contracts::render_content::NeutralMeshV1::bounds)
            .ok_or(ReferenceGameError::PresentationAssetMissing)?;
        Ok::<_, ReferenceGameError>((asset_revision, bounds))
    };
    let (floor_mesh, floor_bounds) = mesh(crate::source::REFERENCE_FLOOR_MESH_ASSET_ID)?;
    let (humanoid_mesh, humanoid_bounds) = mesh(crate::source::REFERENCE_HUMANOID_MESH_ASSET_ID)?;
    let (enemy_mesh, enemy_bounds) = mesh(crate::source::REFERENCE_ENEMY_MESH_ASSET_ID)?;
    let (quest_giver_mesh, quest_giver_bounds) =
        mesh(crate::source::REFERENCE_QUEST_GIVER_MESH_ASSET_ID)?;
    let (blade_mesh, blade_bounds) = mesh(crate::source::REFERENCE_BLADE_MESH_ASSET_ID)?;
    let (relay_mesh, relay_bounds) = mesh(crate::source::REFERENCE_RELAY_MESH_ASSET_ID)?;
    let (relay_approach_mesh, relay_approach_bounds) =
        mesh(crate::source::REFERENCE_RELAY_APPROACH_MESH_ASSET_ID)?;
    let (focus_ring_mesh, focus_ring_bounds) =
        mesh(crate::source::REFERENCE_FOCUS_RING_MESH_ASSET_ID)?;
    let (quest_marker_mesh, quest_marker_bounds) =
        mesh(crate::source::REFERENCE_QUEST_MARKER_MESH_ASSET_ID)?;
    let floor_material = revision(crate::source::REFERENCE_BASE_MATERIAL_ASSET_ID)?;
    let player_material = revision(crate::source::REFERENCE_PLAYER_MATERIAL_ASSET_ID)?;
    let enemy_material = revision(crate::source::REFERENCE_ENEMY_MATERIAL_ASSET_ID)?;
    let defeated_enemy_material =
        revision(crate::source::REFERENCE_DEFEATED_ENEMY_MATERIAL_ASSET_ID)?;
    let quest_giver_material = revision(crate::source::REFERENCE_QUEST_GIVER_MATERIAL_ASSET_ID)?;
    let blade_material = revision(crate::source::REFERENCE_BLADE_MATERIAL_ASSET_ID)?;
    let relay_inactive_material =
        revision(crate::source::REFERENCE_RELAY_INACTIVE_MATERIAL_ASSET_ID)?;
    let relay_active_material = revision(crate::source::REFERENCE_RELAY_ACTIVE_MATERIAL_ASSET_ID)?;
    let relay_approach_material =
        revision(crate::source::REFERENCE_RELAY_APPROACH_MATERIAL_ASSET_ID)?;
    let indicator_material = revision(crate::source::REFERENCE_INDICATOR_MATERIAL_ASSET_ID)?;

    let pickup_collected = matches!(
        aggregate_payload(rpg, RpgAggregateKindV1::Inventory, fixture.player_inventory_id),
        Some(RpgAggregatePayloadV1::Inventory(inventory))
            if inventory.item_ids.contains(&fixture.pickup_item_id)
    );
    let npc_alive =
        match aggregate_payload(rpg, RpgAggregateKindV1::Character, fixture.npc_character_id) {
            Some(RpgAggregatePayloadV1::Character(character)) => character
                .resources
                .iter()
                .find(|resource| resource.resource_id.as_str() == CORE_CHARACTER_HEALTH_RESOURCE_ID)
                .is_none_or(|resource| resource.current_value > 0),
            _ => true,
        };
    let relay_active = matches!(
        aggregate_payload(
            rpg,
            RpgAggregateKindV1::InteractiveObject,
            fixture.interactive_object_id,
        ),
        Some(RpgAggregatePayloadV1::InteractiveObject(relay))
            if relay.state_id.as_str()
                == next_contracts::rpg::CORE_INTERACTIVE_OBJECT_ACTIVATED_STATE_ID
    );
    let relay_material = if relay_active {
        relay_active_material
    } else {
        relay_inactive_material
    };
    let enemy_material = if npc_alive {
        enemy_material
    } else {
        defeated_enemy_material
    };
    let pickup_equipped = matches!(
        aggregate_payload(rpg, RpgAggregateKindV1::Equipment, fixture.player_equipment_id),
        Some(RpgAggregatePayloadV1::Equipment(equipment))
            if equipment
                .assignments
                .iter()
                .any(|assignment| assignment.item_id == fixture.pickup_item_id)
    );
    let quest_definition = fixture
        .activated_project
        .rpg_definitions
        .quests
        .first()
        .ok_or(ReferenceGameError::PresentationAssetMissing)?;
    let quest_state = match aggregate_payload(rpg, RpgAggregateKindV1::Quest, fixture.quest_id) {
        Some(RpgAggregatePayloadV1::Quest(quest)) => Some(&quest.state_id),
        _ => None,
    };
    let quest_complete = quest_state.is_some_and(|state| {
        !quest_definition
            .transitions
            .iter()
            .any(|transition| transition.source_state_id == *state)
    });
    let quest_offer = quest_state == Some(&quest_definition.entry_state_id);
    let focus_body_id = if quest_complete {
        None
    } else if quest_offer {
        Some(PhysicsBodyIdV1 {
            subject_id: fixture.quest_giver_character_id,
            body_slot: 0,
        })
    } else if !pickup_collected {
        Some(PhysicsBodyIdV1 {
            subject_id: fixture.pickup_proxy_id,
            body_slot: 0,
        })
    } else if !pickup_equipped {
        Some(fixture.physics_body_id)
    } else if npc_alive {
        Some(PhysicsBodyIdV1 {
            subject_id: fixture.npc_character_id,
            body_slot: 0,
        })
    } else if !relay_active {
        Some(PhysicsBodyIdV1 {
            subject_id: fixture.interactive_object_id,
            body_slot: 0,
        })
    } else {
        Some(PhysicsBodyIdV1 {
            subject_id: fixture.quest_giver_character_id,
            body_slot: 0,
        })
    };
    let mut bindings = vec![
        PresentationBindingV1 {
            persistent_id: PersistentId::from_bytes([0x57; 16]),
            presentation_role: next_contracts::presentation::PresentationRoleV1::Environment,
            incarnation: 0,
            presentation_layer: 0,
            mesh_revision: floor_mesh,
            material_revision: floor_material,
            instance_ordinal: 0,
            local_bounds: floor_bounds,
            feature_flags: next_contracts::presentation::ScenePresentationFlagsV1::NONE,
            physics_body_id: Some(PhysicsBodyIdV1 {
                subject_id: PersistentId::from_bytes([0x57; 16]),
                body_slot: 0,
            }),
            fallback_transform:
                next_contracts::presentation::QuantizedPresentationTransformV1::default(),
            visible: true,
        },
        PresentationBindingV1 {
            persistent_id: fixture.body_id,
            presentation_role: next_contracts::presentation::PresentationRoleV1::PlayerAvatar,
            incarnation: 0,
            presentation_layer: 1,
            mesh_revision: humanoid_mesh,
            material_revision: player_material,
            instance_ordinal: 0,
            local_bounds: humanoid_bounds,
            feature_flags: next_contracts::presentation::ScenePresentationFlagsV1::NONE,
            physics_body_id: Some(fixture.physics_body_id),
            fallback_transform:
                next_contracts::presentation::QuantizedPresentationTransformV1::default(),
            visible: true,
        },
        PresentationBindingV1 {
            persistent_id: fixture.interactive_object_id,
            presentation_role: next_contracts::presentation::PresentationRoleV1::InteractiveObject,
            incarnation: 0,
            presentation_layer: 2,
            mesh_revision: relay_mesh,
            material_revision: relay_material,
            instance_ordinal: 0,
            local_bounds: relay_bounds,
            feature_flags: next_contracts::presentation::ScenePresentationFlagsV1::NONE,
            physics_body_id: Some(PhysicsBodyIdV1 {
                subject_id: fixture.interactive_object_id,
                body_slot: 0,
            }),
            fallback_transform:
                next_contracts::presentation::QuantizedPresentationTransformV1::default(),
            visible: true,
        },
        PresentationBindingV1 {
            persistent_id: fixture.pickup_item_id,
            presentation_role: next_contracts::presentation::PresentationRoleV1::Item,
            incarnation: 0,
            presentation_layer: 3,
            mesh_revision: blade_mesh,
            material_revision: blade_material,
            instance_ordinal: 0,
            local_bounds: blade_bounds,
            feature_flags: next_contracts::presentation::ScenePresentationFlagsV1::NONE,
            physics_body_id: Some(PhysicsBodyIdV1 {
                subject_id: fixture.pickup_proxy_id,
                body_slot: 0,
            }),
            fallback_transform:
                next_contracts::presentation::QuantizedPresentationTransformV1::default(),
            visible: !pickup_collected,
        },
        PresentationBindingV1 {
            persistent_id: fixture.npc_character_id,
            presentation_role: next_contracts::presentation::PresentationRoleV1::Character,
            incarnation: 0,
            presentation_layer: 4,
            mesh_revision: enemy_mesh,
            material_revision: enemy_material,
            instance_ordinal: 0,
            local_bounds: enemy_bounds,
            feature_flags: next_contracts::presentation::ScenePresentationFlagsV1::NONE,
            physics_body_id: Some(PhysicsBodyIdV1 {
                subject_id: fixture.npc_character_id,
                body_slot: 0,
            }),
            fallback_transform:
                next_contracts::presentation::QuantizedPresentationTransformV1::default(),
            // The current grounded-capsule checkpoint keeps this solid body
            // for exact contact/replay continuation. Keep a darkened defeated
            // body visible so its collider never turns into an invisible
            // obstacle; physical removal belongs to the later topology path.
            visible: true,
        },
        PresentationBindingV1 {
            persistent_id: fixture.quest_giver_character_id,
            presentation_role: next_contracts::presentation::PresentationRoleV1::Character,
            incarnation: 0,
            presentation_layer: 5,
            mesh_revision: quest_giver_mesh,
            material_revision: quest_giver_material,
            instance_ordinal: 0,
            local_bounds: quest_giver_bounds,
            feature_flags: next_contracts::presentation::ScenePresentationFlagsV1::NONE,
            physics_body_id: Some(PhysicsBodyIdV1 {
                subject_id: fixture.quest_giver_character_id,
                body_slot: 0,
            }),
            fallback_transform:
                next_contracts::presentation::QuantizedPresentationTransformV1::default(),
            visible: true,
        },
    ];

    // V1-alpha environment composition is presentation-only. One batched
    // neutral kit piece combines the relay approach and rock field so the
    // visual landmarks do not multiply B0 draw submissions. It comes through
    // the same cooked content and extraction boundary as gameplay-owned actors
    // and never becomes a second source of world or collision truth.
    let static_environment = |persistent_byte,
                              presentation_layer,
                              mesh_revision,
                              material_revision,
                              local_bounds,
                              translation_micrometres,
                              orientation_q30| PresentationBindingV1 {
        persistent_id: PersistentId::from_bytes([persistent_byte; 16]),
        presentation_role: next_contracts::presentation::PresentationRoleV1::Environment,
        incarnation: 0,
        presentation_layer,
        mesh_revision,
        material_revision,
        instance_ordinal: 0,
        local_bounds,
        feature_flags: next_contracts::presentation::ScenePresentationFlagsV1::NONE,
        physics_body_id: None,
        fallback_transform: next_contracts::presentation::QuantizedPresentationTransformV1 {
            translation_micrometres,
            orientation_q30,
        },
        visible: true,
    };
    const IDENTITY_Q30: [i32; 4] = [0, 0, 0, 1 << 30];
    bindings.push(static_environment(
        0x70,
        10,
        relay_approach_mesh,
        relay_approach_material,
        relay_approach_bounds,
        [0, 0, 0],
        IDENTITY_Q30,
    ));
    if let Some(focus_body_id) = focus_body_id {
        bindings.push(PresentationBindingV1 {
            persistent_id: PersistentId::from_bytes([0x75; 16]),
            presentation_role: next_contracts::presentation::PresentationRoleV1::Environment,
            incarnation: 0,
            presentation_layer: 240,
            mesh_revision: focus_ring_mesh,
            material_revision: indicator_material,
            instance_ordinal: 240,
            local_bounds: focus_ring_bounds,
            feature_flags: next_contracts::presentation::ScenePresentationFlagsV1::NONE,
            physics_body_id: Some(focus_body_id),
            fallback_transform:
                next_contracts::presentation::QuantizedPresentationTransformV1::default(),
            visible: true,
        });
        if !quest_offer {
            bindings.push(PresentationBindingV1 {
                persistent_id: PersistentId::from_bytes([0x76; 16]),
                presentation_role: next_contracts::presentation::PresentationRoleV1::Environment,
                incarnation: 0,
                presentation_layer: 241,
                mesh_revision: quest_marker_mesh,
                material_revision: indicator_material,
                instance_ordinal: 241,
                local_bounds: quest_marker_bounds,
                feature_flags: next_contracts::presentation::ScenePresentationFlagsV1::NONE,
                physics_body_id: Some(focus_body_id),
                fallback_transform:
                    next_contracts::presentation::QuantizedPresentationTransformV1::default(),
                visible: true,
            });
        }
    }
    Ok(bindings)
}
