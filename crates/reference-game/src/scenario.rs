use next_contracts::cognition::{AgentCognitionSnapshotV1, AgentMemorySnapshotV1, DecisionTraceV1};
use next_contracts::ids::{ContentHash, PersistentId, SchemaId, StateRoot};
use next_contracts::input::{CORE_MELEE_ACTION_ID, PlayerActionPhaseV1};
use next_contracts::mechanics::CORE_CHARACTER_HEALTH_RESOURCE_ID;
use next_contracts::physics::{PhysicsBodyIdV1, PhysicsPoseV1};
use next_contracts::rpg::{RpgAggregateKindV1, RpgAggregatePayloadV1, RpgSnapshotV2};
use next_contracts::world_activity::WorldActivitySnapshotV1;
use next_contracts::world_population::WorldPopulationSnapshotV1;
use next_contracts::world_routine::{
    InteractionAvailabilityV1, WorldRoutineActivityV1, WorldRoutineSnapshotV1,
};
use next_presentation::PresentationBindingV1;
use next_runtime::{PhysicsLaunchOptions, RuntimeState, WorldServicesTickCommitV1};
use next_world::{
    PreparedWorldStreamingPublicationV1, WorldActivityOwnerV1, WorldPopulationOwnerV1,
    WorldRoutineOwnerV1, WorldStreamerV1,
};

use crate::ReferenceGameError;
use crate::input::NormalizedReferenceInputV1;
use crate::rpg::{
    aggregate_payload, cognition_only_rpg_snapshot, cooked_interaction_outcome,
    cooked_project_rpg_snapshot,
};
use crate::session::{
    ReferenceGameSession, build_reference_game_session, build_reference_game_session_with_profile,
};

mod evidence;
mod presentation;

use evidence::{accumulate_report, reference_stage_checkpoint, rpg_contact_facts_from_report};
pub(super) use presentation::fixture_presentation_bindings;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceStageCheckpointV2 {
    pub state_root: StateRoot,
    pub application_state_root: StateRoot,
    pub world_routine_activity_or_none: Option<WorldRoutineActivityV1>,
    pub world_routine_record_revision_or_none: Option<u64>,
    pub quest_state_id: SchemaId,
    pub dialogue_node_id: SchemaId,
    pub player_inventory_contains_pickup: bool,
    pub player_equipment_contains_pickup: bool,
    pub npc_health: i32,
    pub player_health: i32,
    pub relay_state_id: SchemaId,
    pub current_chunk_id: SchemaId,
}

pub struct ReferenceRunOutcomeV2 {
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
    pub world_routine_snapshot_or_none: Option<WorldRoutineSnapshotV1>,
    pub world_population_snapshot: WorldPopulationSnapshotV1,
    pub world_activity_snapshot_or_none: Option<WorldActivitySnapshotV1>,
    pub agent_cognition_snapshot: AgentCognitionSnapshotV1,
    pub agent_memory_snapshot: AgentMemorySnapshotV1,
    pub decision_traces: Vec<DecisionTraceV1>,
    pub world_services_tick_commits: Vec<WorldServicesTickCommitV1>,
    pub world_routine_rest_branch_or_none: Option<ReferenceWorldRoutineRestBranchV1>,
    pub agent_intent_id: Option<ContentHash>,
    pub agent_projection_hash: Option<ContentHash>,
    pub stage_checkpoint_roots: Vec<StateRoot>,
    pub stage_checkpoints: Vec<ReferenceStageCheckpointV2>,
}

/// Exact isolated branch-B evidence for `WORLD-ROUTINE-P1`. The branch starts
/// from the same authored Duty/quest-available state as the main branch-A run,
/// crosses the boundary without accepting, queries availability in Rest and
/// submits the same production interaction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceWorldRoutineRestBranchV1 {
    pub initial_rpg_snapshot: RpgSnapshotV2,
    pub rest_query: InteractionAvailabilityV1,
    pub world_services_tick_commits: Vec<WorldServicesTickCommitV1>,
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

enum PendingPackagedTransitionV1 {
    Begin {
        publication: PreparedWorldStreamingPublicationV1,
        save_restore: bool,
    },
    Complete {
        publication: PreparedWorldStreamingPublicationV1,
    },
}

pub fn run_reference_game(
    package: next_project::ActivatedProjectPackage,
    include_interaction: bool,
) -> Result<ReferenceRunOutcomeV2, ReferenceGameError> {
    run_reference_game_with_backend(
        include_interaction,
        false,
        PhysicsLaunchOptions::default(),
        package,
    )
}

pub fn run_reference_game_with_backend(
    include_interaction: bool,
    physx_compatible: bool,
    physics_options: PhysicsLaunchOptions,
    package: next_project::ActivatedProjectPackage,
) -> Result<ReferenceRunOutcomeV2, ReferenceGameError> {
    let next_project::ActivatedProjectPackage {
        project: activated_project,
        content_generation,
    } = package;
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
        cognition_only_rpg_snapshot(&fixture)
    };
    let world_routine_rest_branch_or_none = if include_interaction {
        let first = run_world_routine_rest_branch(
            &fixture,
            &content_generation,
            rpg_snapshot.clone(),
            physics_options,
        )?;
        let repeated = run_world_routine_rest_branch(
            &fixture,
            &content_generation,
            rpg_snapshot.clone(),
            physics_options,
        )?;
        if first != repeated {
            return Err(ReferenceGameError::RecoveryInvalid);
        }
        Some(first)
    } else {
        None
    };
    let mut runtime = RuntimeState::with_rpg_snapshot_and_physics_options(
        fixture.bootstrap.clone(),
        fixture.authority.clone(),
        rpg_snapshot,
        physics_options,
    )?;
    let mut input_producer = NormalizedReferenceInputV1::new(&fixture)?;
    let initial_chunk_id = fixture.world_topology().initial_chunk_id().clone();
    let transition_chunk_id = fixture.world_topology().gameplay_target_chunk_id().clone();
    let mut world_streamer = WorldStreamerV1::activate(
        fixture.activated_project.clone(),
        content_generation.clone(),
        initial_chunk_id.clone(),
    )?;
    let mut world_routine = WorldRoutineOwnerV1::activate(
        fixture.activated_project.world_routine_catalog_or_none,
        runtime.next_tick(),
    )?;
    let mut world_population = WorldPopulationOwnerV1::activate(
        fixture.activated_project.world_population_catalog.clone(),
        fixture.activated_project.world_navigation_catalog.clone(),
        runtime.next_tick(),
    )?;
    let mut cognition = fixture.initial_cognition_owners()?;
    let mut activity = include_interaction
        .then(|| fixture.initial_activity_owner())
        .transpose()?;
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
    let mut world_services_tick_commits: Vec<WorldServicesTickCommitV1> = Vec::new();
    let mut sequence = 0_u64;
    let mut agent_intent_id = None;
    let mut agent_projection_hash = None;
    let mut stage_checkpoint_roots = Vec::new();
    let mut stage_checkpoints = Vec::new();
    let mut pending_packaged_transition = None;
    for action in inputs {
        if matches!(&action, ScenarioAction::Checkpoint) {
            let checkpoint = runtime.world_checkpoint()?;
            let expected_root = checkpoint.state_root;
            let routine_snapshot_or_none = world_routine.snapshot_or_none().copied();
            let population_snapshot = world_population
                .snapshot_or_none()
                .cloned()
                .ok_or(ReferenceGameError::RecoveryInvalid)?;
            let agent_snapshot = cognition.agent_snapshot().clone();
            let memory_snapshot = cognition.memory_snapshot().clone();
            let activity_snapshot_or_none = activity.as_ref().map(|owner| owner.snapshot().clone());
            let expected_application_root = world_services_tick_commits
                .last()
                .ok_or(ReferenceGameError::RecoveryInvalid)?
                .application_state_root;
            runtime = RuntimeState::restore_world_checkpoint_with_definitions_and_physics_options(
                checkpoint,
                fixture.authority.clone(),
                fixture.activated_project.rpg_definitions.clone(),
                physics_options,
            )?;
            world_routine = WorldRoutineOwnerV1::restore(
                fixture.activated_project.world_routine_catalog_or_none,
                routine_snapshot_or_none,
                runtime.next_tick(),
            )?;
            world_population = WorldPopulationOwnerV1::restore(
                fixture.activated_project.world_population_catalog.clone(),
                fixture.activated_project.world_navigation_catalog.clone(),
                population_snapshot,
                runtime.next_tick(),
            )?;
            cognition = next_agent::cognition::StrategicAgentOwnersV1::restore(
                fixture.activated_project.agent_cognition_catalog.clone(),
                agent_snapshot,
                memory_snapshot,
            )?;
            activity = activity_snapshot_or_none
                .map(|snapshot| {
                    WorldActivityOwnerV1::restore(
                        fixture.activated_project.world_activity_catalog.clone(),
                        snapshot,
                        runtime.next_tick(),
                    )
                })
                .transpose()?;
            runtime.validate_world_routine_ledger_closure(&world_routine)?;
            runtime.validate_world_population_ledger_closure(&world_population)?;
            let restored_root = runtime.world_checkpoint()?.state_root;
            if restored_root != expected_root {
                return Err(ReferenceGameError::RecoveryInvalid);
            }
            stage_checkpoint_roots.push(restored_root);
            stage_checkpoints.push(reference_stage_checkpoint(
                &runtime,
                &world_routine,
                &fixture,
                restored_root,
                expected_application_root,
                world_streamer.snapshot().current_chunk_id.clone(),
            )?);
            continue;
        }
        if let ScenarioAction::ChunkTransition(target_chunk_id, save_restore) = action {
            if pending_packaged_transition.is_some() {
                return Err(ReferenceGameError::WorldStreamingResumeMismatch);
            }
            let publication =
                world_streamer.prepare_begin_transition(target_chunk_id, runtime.next_tick())?;
            pending_packaged_transition = Some(PendingPackagedTransitionV1::Begin {
                publication,
                save_restore,
            });
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
            let commit = run_scenario_tick(
                &mut runtime,
                ScenarioWorldServices {
                    routine: &mut world_routine,
                    population: &mut world_population,
                    activity: activity.as_mut(),
                    cognition: &mut cognition,
                    world: &mut world_streamer,
                },
                &fixture.activated_project,
                &content_generation,
                [planned.world_command],
                &mut pending_packaged_transition,
            )?;
            let report = commit.runtime_report.clone();
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
            world_services_tick_commits.push(commit);
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
        let commit = run_scenario_tick(
            &mut runtime,
            ScenarioWorldServices {
                routine: &mut world_routine,
                population: &mut world_population,
                activity: activity.as_mut(),
                cognition: &mut cognition,
                world: &mut world_streamer,
            },
            &fixture.activated_project,
            &content_generation,
            [],
            &mut pending_packaged_transition,
        )?;
        let report = commit.runtime_report.clone();
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
        world_services_tick_commits.push(commit);
        sequence = sequence
            .checked_add(1)
            .ok_or(ReferenceGameError::CountOverflow)?;
    }
    if pending_packaged_transition.is_some() {
        return Err(ReferenceGameError::WorldStreamingResumeMismatch);
    }
    let final_pose = runtime
        .physics_snapshot()
        .sorted_body_states
        .get(&fixture.physics_body_id)
        .ok_or(ReferenceGameError::BodyMissing)?
        .pose;
    let ticks = runtime.next_tick();
    let world_streaming_snapshot = world_streamer.snapshot();
    let world_routine_snapshot_or_none = world_routine.snapshot_or_none().copied();
    let world_population_snapshot = world_population
        .snapshot_or_none()
        .cloned()
        .ok_or(ReferenceGameError::RecoveryInvalid)?;
    let world_activity_snapshot_or_none = activity.as_ref().map(|owner| owner.snapshot().clone());
    let agent_cognition_snapshot = cognition.agent_snapshot().clone();
    let agent_memory_snapshot = cognition.memory_snapshot().clone();
    let decision_traces = world_services_tick_commits
        .iter()
        .filter_map(|commit| commit.decision_trace_or_none.clone())
        .collect();
    let presentation_bindings = fixture_presentation_bindings(&fixture, &runtime.rpg_snapshot())?;
    Ok(ReferenceRunOutcomeV2 {
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
        project_composition_lock_hash: fixture.activated_project.project_lock.project_lock_sha256,
        content_manifest_hash: fixture
            .activated_project
            .content_manifest
            .content_manifest_sha256,
        render_content_catalog: fixture.activated_project.render_content_catalog.clone(),
        presentation_bindings,
        tick_reports,
        world_streaming_snapshot: world_streaming_snapshot.clone(),
        world_routine_snapshot_or_none,
        world_population_snapshot,
        world_activity_snapshot_or_none,
        agent_cognition_snapshot,
        agent_memory_snapshot,
        decision_traces,
        world_services_tick_commits,
        world_routine_rest_branch_or_none,
        agent_intent_id,
        agent_projection_hash,
        stage_checkpoint_roots,
        stage_checkpoints,
    })
}

fn run_world_routine_rest_branch(
    fixture: &ReferenceGameSession,
    content_generation: &next_assets::PinnedContentGeneration,
    initial_rpg_snapshot: RpgSnapshotV2,
    physics_options: PhysicsLaunchOptions,
) -> Result<ReferenceWorldRoutineRestBranchV1, ReferenceGameError> {
    let mut runtime = RuntimeState::with_rpg_snapshot_and_physics_options(
        fixture.bootstrap.clone(),
        fixture.authority.clone(),
        initial_rpg_snapshot.clone(),
        physics_options,
    )?;
    let mut world = WorldStreamerV1::activate(
        fixture.activated_project.clone(),
        content_generation.clone(),
        fixture.world_topology().initial_chunk_id().clone(),
    )?;
    let mut routine = WorldRoutineOwnerV1::activate(
        fixture.activated_project.world_routine_catalog_or_none,
        runtime.next_tick(),
    )?;
    let mut population = WorldPopulationOwnerV1::activate(
        fixture.activated_project.world_population_catalog.clone(),
        fixture.activated_project.world_navigation_catalog.clone(),
        runtime.next_tick(),
    )?;
    let mut cognition = fixture.initial_cognition_owners()?;
    let mut commits = Vec::with_capacity(4);
    for expected_tick in 0_u64..=2 {
        let prepared = runtime
            .tick_preparation()
            .prepare_with_world_services_and_cognition(
                [],
                &routine,
                &population,
                &cognition,
                &world,
            )?;
        let validated = runtime.validate_prepared_world_services_tick_with_cognition(
            &routine,
            &population,
            &cognition,
            &world,
            prepared,
        )?;
        let commit = runtime.commit_validated_world_services_tick_with_cognition(
            &mut routine,
            &mut population,
            &mut cognition,
            &mut world,
            validated,
        )?;
        if commit.runtime_report.tick != expected_tick {
            return Err(ReferenceGameError::RecoveryInvalid);
        }
        commits.push(commit);
    }
    let interaction_id = fixture
        .activated_project
        .rpg_definitions
        .interactions
        .first()
        .ok_or(ReferenceGameError::RecoveryInvalid)?
        .interaction_id
        .clone();
    let rest_query = runtime.interaction_availability(&interaction_id, &routine)?;
    runtime.enqueue_input_sample(
        &fixture.principal,
        crate::input::player_interact_sample(fixture, 0, PlayerActionPhaseV1::Started, true, None)?,
    )?;
    let prepared = runtime
        .tick_preparation()
        .prepare_with_world_services_and_cognition([], &routine, &population, &cognition, &world)?;
    let validated = runtime.validate_prepared_world_services_tick_with_cognition(
        &routine,
        &population,
        &cognition,
        &world,
        prepared,
    )?;
    commits.push(runtime.commit_validated_world_services_tick_with_cognition(
        &mut routine,
        &mut population,
        &mut cognition,
        &mut world,
        validated,
    )?);
    Ok(ReferenceWorldRoutineRestBranchV1 {
        initial_rpg_snapshot,
        rest_query,
        world_services_tick_commits: commits,
    })
}

struct ScenarioWorldServices<'a> {
    routine: &'a mut WorldRoutineOwnerV1,
    population: &'a mut WorldPopulationOwnerV1,
    activity: Option<&'a mut WorldActivityOwnerV1>,
    cognition: &'a mut next_agent::cognition::StrategicAgentOwnersV1,
    world: &'a mut WorldStreamerV1,
}

fn run_scenario_tick(
    runtime: &mut RuntimeState,
    services: ScenarioWorldServices<'_>,
    project: &next_contracts::project::ActivatedProjectV6,
    content_generation: &next_assets::PinnedContentGeneration,
    commands: impl IntoIterator<Item = next_contracts::command::WorldCommand>,
    pending: &mut Option<PendingPackagedTransitionV1>,
) -> Result<WorldServicesTickCommitV1, ReferenceGameError> {
    let ScenarioWorldServices {
        routine,
        population,
        activity,
        cognition,
        world,
    } = services;
    let Some(stage) = pending.take() else {
        return if let Some(activity) = activity {
            let prepared = runtime
                .tick_preparation()
                .prepare_with_world_services_cognition_and_activity(
                    commands, routine, population, activity, cognition, world,
                )?;
            let validated = runtime
                .validate_prepared_world_services_tick_with_cognition_and_activity(
                    routine, population, activity, cognition, world, prepared,
                )?;
            Ok(
                runtime.commit_validated_world_services_tick_with_cognition_and_activity(
                    routine, population, activity, cognition, world, validated,
                )?,
            )
        } else {
            let prepared = runtime
                .tick_preparation()
                .prepare_with_world_services_and_cognition(
                    commands, routine, population, cognition, world,
                )?;
            let validated = runtime.validate_prepared_world_services_tick_with_cognition(
                routine, population, cognition, world, prepared,
            )?;
            Ok(runtime.commit_validated_world_services_tick_with_cognition(
                routine, population, cognition, world, validated,
            )?)
        };
    };
    match stage {
        PendingPackagedTransitionV1::Begin {
            publication,
            save_restore,
        } => {
            let commit = if let Some(activity) = activity {
                let prepared = runtime
                    .tick_preparation()
                    .prepare_with_world_services_cognition_activity_and_streaming(
                        commands,
                        routine,
                        population,
                        activity,
                        cognition,
                        world,
                        publication,
                    )?;
                let validated = runtime
                    .validate_prepared_world_services_tick_with_cognition_and_activity(
                        routine, population, activity, cognition, world, prepared,
                    )?;
                runtime.commit_validated_world_services_tick_with_cognition_and_activity(
                    routine, population, activity, cognition, world, validated,
                )?
            } else {
                let prepared = runtime
                    .tick_preparation()
                    .prepare_with_world_services_cognition_and_streaming(
                        commands,
                        routine,
                        population,
                        cognition,
                        world,
                        publication,
                    )?;
                let validated = runtime.validate_prepared_world_services_tick_with_cognition(
                    routine, population, cognition, world, prepared,
                )?;
                runtime.commit_validated_world_services_tick_with_cognition(
                    routine, population, cognition, world, validated,
                )?
            };
            debug_assert!(commit.streaming_transition_or_none.is_none());
            if save_restore {
                let saved = world.snapshot().canonical_bytes()?;
                let decoded =
                    next_contracts::world::WorldStreamingSnapshotV1::from_canonical_bytes(
                        &saved,
                        next_contracts::canonical::CanonicalDecodeLimits::default(),
                    )?;
                *world =
                    WorldStreamerV1::restore(project.clone(), content_generation.clone(), decoded)?;
            }
            // Mandatory I/O runs while simulation advancement is paused. Its
            // completion is bound to the already selected next gameplay tick.
            let loaded = world.load_pending(next_world::WORLD_CHUNK_DEFAULT_WORKERS)?;
            let publication = world.prepare_loaded_commit(loaded, runtime.next_tick())?;
            *pending = Some(PendingPackagedTransitionV1::Complete { publication });
            Ok(commit)
        }
        PendingPackagedTransitionV1::Complete { publication } => {
            let commit = if let Some(activity) = activity {
                let prepared = runtime
                    .tick_preparation()
                    .prepare_with_world_services_cognition_activity_and_streaming(
                        commands,
                        routine,
                        population,
                        activity,
                        cognition,
                        world,
                        publication,
                    )?;
                let validated = runtime
                    .validate_prepared_world_services_tick_with_cognition_and_activity(
                        routine, population, activity, cognition, world, prepared,
                    )?;
                runtime.commit_validated_world_services_tick_with_cognition_and_activity(
                    routine, population, activity, cognition, world, validated,
                )?
            } else {
                let prepared = runtime
                    .tick_preparation()
                    .prepare_with_world_services_cognition_and_streaming(
                        commands,
                        routine,
                        population,
                        cognition,
                        world,
                        publication,
                    )?;
                let validated = runtime.validate_prepared_world_services_tick_with_cognition(
                    routine, population, cognition, world, prepared,
                )?;
                runtime.commit_validated_world_services_tick_with_cognition(
                    routine, population, cognition, world, validated,
                )?
            };
            if commit.streaming_transition_or_none.is_none() {
                return Err(ReferenceGameError::WorldStreamingResumeMismatch);
            }
            Ok(commit)
        }
    }
}
