use next_contracts::command::EventPayload;
use next_contracts::ids::{ContentHash, PersistentId, SchemaId};
use next_contracts::input::{CORE_MELEE_ACTION_ID, PlayerActionPhaseV1};
use next_contracts::physics::{ContactPhaseV1, PhysicsBodyIdV1, PhysicsPoseV1};
use next_contracts::project::ActivatedProjectV2;
use next_contracts::rpg::{RpgPhysicalContactFactV1, RpgSnapshotV2};
use next_presentation::PresentationBindingV1;
use next_runtime::{PhysicsLaunchOptions, RuntimeState};
use next_world::WorldStreamerV1;

use crate::rpg::{cooked_interaction_outcome, cooked_project_rpg_snapshot};
use crate::session::{
    ReferenceGameSession, build_reference_game_session, build_reference_game_session_with_profile,
};
use crate::{
    ReferenceGameError, player_action_sample, player_equip_use_sample, player_interact_sample,
    player_melee_sample, player_pickup_sample,
};

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
    pub player_character_id: PersistentId,
    pub dialogue_id: PersistentId,
    pub quest_id: PersistentId,
    pub relationship_id: PersistentId,
    pub relationship_dimension_id: SchemaId,
    pub project_composition_lock_hash: ContentHash,
    pub content_manifest_hash: ContentHash,
    pub presentation_bindings: Vec<PresentationBindingV1>,
    pub tick_reports: Vec<next_runtime::TickReport>,
    pub world_streaming_snapshot: next_contracts::world::WorldStreamingSnapshotV1,
    pub agent_intent_id: Option<ContentHash>,
    pub agent_projection_hash: Option<ContentHash>,
}

enum ScenarioAction {
    Movement(PlayerActionPhaseV1, [i16; 2]),
    Interaction,
    Pickup,
    EquipUse,
    Melee,
    AgentMelee,
    ChunkTransition(SchemaId, bool),
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
    let mut inputs = vec![
        ScenarioAction::Movement(PlayerActionPhaseV1::Started, [0, 32_767]),
        ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [0, 32_767]),
        ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [0, 32_767]),
        ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [0, 32_767]),
    ];
    if include_interaction {
        inputs.extend([
            ScenarioAction::Pickup,
            ScenarioAction::EquipUse,
            ScenarioAction::ChunkTransition(transition_chunk_id, true),
            ScenarioAction::Interaction,
        ]);
        inputs.extend([
            ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [0, -32_767]),
            ScenarioAction::Movement(PlayerActionPhaseV1::Started, [32_767, 0]),
            ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [32_767, 0]),
            ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [32_767, 0]),
            ScenarioAction::Melee,
            ScenarioAction::AgentMelee,
            ScenarioAction::Interaction,
            ScenarioAction::ChunkTransition(initial_chunk_id, false),
            ScenarioAction::Movement(PlayerActionPhaseV1::Performed, [-32_767, 0]),
            ScenarioAction::Movement(PlayerActionPhaseV1::Completed, [0, 0]),
        ]);
    } else {
        inputs.extend([
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
    for action in inputs {
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
        let wall_time =
            Some(i64::try_from(sequence).map_err(|_| ReferenceGameError::CountOverflow)? * 1000);
        let sample = match action {
            ScenarioAction::Movement(phase, direction) => {
                player_action_sample(&fixture, sequence, phase, direction, wall_time)?
            }
            ScenarioAction::Interaction => player_interact_sample(
                &fixture,
                sequence,
                PlayerActionPhaseV1::Started,
                true,
                wall_time,
            )?,
            ScenarioAction::Pickup => player_pickup_sample(
                &fixture,
                sequence,
                PlayerActionPhaseV1::Started,
                true,
                wall_time,
            )?,
            ScenarioAction::EquipUse => player_equip_use_sample(
                &fixture,
                sequence,
                PlayerActionPhaseV1::Started,
                true,
                wall_time,
            )?,
            ScenarioAction::Melee => player_melee_sample(
                &fixture,
                sequence,
                PlayerActionPhaseV1::Started,
                true,
                wall_time,
            )?,
            ScenarioAction::AgentMelee => unreachable!("handled before input mapping"),
            ScenarioAction::ChunkTransition(_, _) => unreachable!("handled before input mapping"),
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
        player_character_id: fixture.body_id,
        dialogue_id: fixture.dialogue_id,
        quest_id: fixture.quest_id,
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
        presentation_bindings: fixture_presentation_bindings(&fixture)?,
        tick_reports,
        world_streaming_snapshot: world_streaming_snapshot.clone(),
        agent_intent_id,
        agent_projection_hash,
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

fn fixture_presentation_bindings(
    fixture: &ReferenceGameSession,
) -> Result<Vec<PresentationBindingV1>, ReferenceGameError> {
    let asset = |kind: next_contracts::content::NeutralRecordKindV1| {
        fixture
            .activated_project
            .neutral_records
            .iter()
            .find(|record| record.kind == kind)
            .map(|record| record.asset_id)
            .ok_or(ReferenceGameError::PresentationAssetMissing)
    };
    Ok(vec![
        PresentationBindingV1 {
            persistent_id: PersistentId::from_bytes([0x57; 16]),
            presentation_role: next_contracts::presentation::PresentationRoleV1::Environment,
            incarnation: 0,
            presentation_layer: 0,
            asset_id: asset(next_contracts::content::NeutralRecordKindV1::Scene)?,
            instance_ordinal: 0,
            primitive: next_contracts::presentation::PresentationPrimitiveV1::Floor,
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
            asset_id: asset(next_contracts::content::NeutralRecordKindV1::Collider)?,
            instance_ordinal: 0,
            primitive: next_contracts::presentation::PresentationPrimitiveV1::Capsule,
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
            asset_id: asset(next_contracts::content::NeutralRecordKindV1::InteractionDefinition)?,
            instance_ordinal: 0,
            primitive: next_contracts::presentation::PresentationPrimitiveV1::Switch,
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
            asset_id: asset(next_contracts::content::NeutralRecordKindV1::ItemDefinition)?,
            instance_ordinal: 0,
            primitive: next_contracts::presentation::PresentationPrimitiveV1::Item,
            physics_body_id: Some(PhysicsBodyIdV1 {
                subject_id: fixture.pickup_proxy_id,
                body_slot: 0,
            }),
            fallback_transform:
                next_contracts::presentation::QuantizedPresentationTransformV1::default(),
            visible: true,
        },
        PresentationBindingV1 {
            persistent_id: fixture.npc_character_id,
            presentation_role: next_contracts::presentation::PresentationRoleV1::Character,
            incarnation: 0,
            presentation_layer: 4,
            asset_id: asset(next_contracts::content::NeutralRecordKindV1::CharacterDefinition)?,
            instance_ordinal: 0,
            primitive: next_contracts::presentation::PresentationPrimitiveV1::Character,
            physics_body_id: Some(PhysicsBodyIdV1 {
                subject_id: fixture.npc_character_id,
                body_slot: 0,
            }),
            fallback_transform:
                next_contracts::presentation::QuantizedPresentationTransformV1::default(),
            visible: true,
        },
    ])
}
