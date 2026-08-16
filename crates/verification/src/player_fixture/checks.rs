use std::path::Path;

use next_contracts::ids::{CommandLedgerHash, ContentHash, SchemaId, StateRoot};
use next_contracts::localization::TextCatalogV1;
use next_contracts::mechanics::CORE_CHARACTER_HEALTH_RESOURCE_ID;
use next_contracts::physics::PhysicsPoseV1;
use next_contracts::project::domain_hash;
use next_contracts::rpg::{RpgAggregateKindV1, RpgAggregatePayloadV1, RpgSnapshotV2};
use next_physics_api::PhysicsBackendPolicy;
use next_presentation::PresentationExtractorV1;
use next_render::{ReferenceB0Renderer, RenderDevice, RenderTargetV1};
use next_runtime::PhysicsLaunchOptions;

use crate::compute_world_checkpoint_root;
use crate::scratch::ScratchContext;

use super::error::PlayCheckError;
use super::prepare_fixture_project_package_with_scratch;
use next_reference_game::{
    ReferenceRunOutcomeV2, aggregate_payload, run_reference_game, run_reference_game_with_backend,
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
    pub indexed_draw_count: u32,
    pub fallback_material_draw_count: u32,
    pub frame_plan_hash: ContentHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedGameFrameV1 {
    pub check: GameCheckReport,
    pub snapshot: next_contracts::presentation::PresentationSnapshotV2,
    pub render_content_catalog: next_contracts::render_content::RenderContentCatalogV1,
    pub text_catalogs: Vec<TextCatalogV1>,
}

pub fn run_play_check() -> Result<PlayCheckReport, PlayCheckError> {
    run_play_check_in(&std::env::temp_dir())
}

pub fn run_play_check_in(scratch_root: &Path) -> Result<PlayCheckReport, PlayCheckError> {
    let scratch = ScratchContext::new(scratch_root).map_err(scratch_error)?;
    run_play_check_with_scratch(&scratch)
}

pub(crate) fn run_play_check_with_scratch(
    scratch: &ScratchContext,
) -> Result<PlayCheckReport, PlayCheckError> {
    let prepared = prepare_fixture_project_package_with_scratch(
        scratch,
        next_reference_game::REFERENCE_GAME_PROJECT_ID,
    )?;
    let result = run_reference_game(prepared.package.clone(), true)
        .map_err(PlayCheckError::from)
        .and_then(play_check_report);
    prepared.finish(result, scratch_error)
}

pub fn run_game_check() -> Result<GameCheckReport, PlayCheckError> {
    Ok(prepare_game_frame()?.check)
}

pub fn prepare_game_frame() -> Result<PreparedGameFrameV1, PlayCheckError> {
    prepare_game_frame_in(&std::env::temp_dir())
}

pub fn prepare_game_frame_in(scratch_root: &Path) -> Result<PreparedGameFrameV1, PlayCheckError> {
    let scratch = ScratchContext::new(scratch_root).map_err(scratch_error)?;
    prepare_game_frame_with_scratch(&scratch)
}

pub(crate) fn prepare_game_frame_with_scratch(
    scratch: &ScratchContext,
) -> Result<PreparedGameFrameV1, PlayCheckError> {
    let prepared = prepare_fixture_project_package_with_scratch(
        scratch,
        next_reference_game::REFERENCE_GAME_PROJECT_ID,
    )?;
    let text_catalogs = prepared.package.project.text_catalogs.clone();
    let result = run_reference_game(prepared.package.clone(), true)
        .map_err(PlayCheckError::from)
        .and_then(|scenario| prepare_game_frame_from_scenario(scenario, text_catalogs));
    prepared.finish(result, scratch_error)
}

pub fn prepare_game_frame_with_activated_project(
    package: next_project::ActivatedProjectPackage,
) -> Result<PreparedGameFrameV1, PlayCheckError> {
    let text_catalogs = package.project.text_catalogs.clone();
    let scenario =
        run_reference_game_with_backend(true, false, PhysicsLaunchOptions::default(), package)?;
    prepare_game_frame_from_scenario(scenario, text_catalogs)
}

fn prepare_game_frame_from_scenario(
    scenario: ReferenceRunOutcomeV2,
    text_catalogs: Vec<TextCatalogV1>,
) -> Result<PreparedGameFrameV1, PlayCheckError> {
    let mut extractor = PresentationExtractorV1::new(
        scenario.project_composition_lock_hash,
        domain_hash(
            "nextengine.presentation-profile.b0.v1",
            b"sdr-reference-no-optional-features",
        ),
        8,
    )?;
    let ui_records = next_reference_game::read_only_screen_semantic_ui_records_for_ids(
        extractor.snapshot_epoch(),
        scenario.player_character_id,
        scenario.quest_id,
        &[scenario.pickup_item_id, scenario.npc_weapon_item_id],
        &scenario.item_display_text_id,
        &scenario.quest_display_text_id,
        &scenario.runtime.rpg_snapshot(),
    )?;
    let snapshot = extractor
        .extract_with_cameras_and_semantic_ui(
            scenario.ticks,
            scenario.project_composition_lock_hash,
            scenario.content_manifest_hash,
            scenario.runtime.physics_snapshot(),
            &scenario.presentation_bindings,
            &[],
            ui_records,
        )?
        .clone();
    let render_content_catalog = scenario.render_content_catalog.clone();
    let mut renderer = ReferenceB0Renderer::new(render_content_catalog.clone())?;
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
        indexed_draw_count: frame.indexed_draw_count,
        fallback_material_draw_count: frame.fallback_material_draw_count,
        frame_plan_hash: frame.frame_plan_hash,
    };
    Ok(PreparedGameFrameV1 {
        check,
        snapshot,
        render_content_catalog,
        text_catalogs,
    })
}

pub fn run_play_check_with_activated_project(
    package: next_project::ActivatedProjectPackage,
) -> Result<PlayCheckReport, PlayCheckError> {
    let scenario =
        run_reference_game_with_backend(true, false, PhysicsLaunchOptions::default(), package)?;
    play_check_report(scenario)
}

fn play_check_report(scenario: ReferenceRunOutcomeV2) -> Result<PlayCheckReport, PlayCheckError> {
    let stage_checkpoint_count = scenario.stage_checkpoint_roots.len();
    let stage_checkpoints_match_acceptance = match scenario.stage_checkpoints.as_slice() {
        [accepted, combat, relay] => {
            accepted.state_root == scenario.stage_checkpoint_roots[0]
                && combat.state_root == scenario.stage_checkpoint_roots[1]
                && relay.state_root == scenario.stage_checkpoint_roots[2]
                && accepted.quest_state_id.as_str() == "nextengine.reference-alpha.quest.active"
                && accepted.dialogue_node_id.as_str()
                    == "nextengine.reference-alpha.dialogue.accepted"
                && !accepted.player_inventory_contains_pickup
                && !accepted.player_equipment_contains_pickup
                && accepted.npc_health == 100
                && accepted.player_health == 100
                && accepted.relay_state_id.as_str()
                    == next_contracts::rpg::CORE_INTERACTIVE_OBJECT_READY_STATE_ID
                && accepted.world_routine_activity_or_none
                    == Some(next_contracts::world_routine::WorldRoutineActivityV1::Duty)
                && accepted.world_routine_record_revision_or_none == Some(0)
                && combat.quest_state_id == accepted.quest_state_id
                && combat.dialogue_node_id == accepted.dialogue_node_id
                && combat.player_inventory_contains_pickup
                && combat.player_equipment_contains_pickup
                && combat.npc_health == 0
                && combat.player_health == 50
                && combat.relay_state_id.as_str()
                    == next_contracts::rpg::CORE_INTERACTIVE_OBJECT_READY_STATE_ID
                && combat.current_chunk_id != accepted.current_chunk_id
                && combat.world_routine_activity_or_none
                    == Some(next_contracts::world_routine::WorldRoutineActivityV1::Rest)
                && combat.world_routine_record_revision_or_none == Some(1)
                && relay.quest_state_id == accepted.quest_state_id
                && relay.dialogue_node_id == accepted.dialogue_node_id
                && relay.player_inventory_contains_pickup
                && relay.player_equipment_contains_pickup
                && relay.npc_health == 0
                && relay.player_health == 50
                && relay.relay_state_id.as_str()
                    == next_contracts::rpg::CORE_INTERACTIVE_OBJECT_ACTIVATED_STATE_ID
                && relay.current_chunk_id == combat.current_chunk_id
                && relay.world_routine_activity_or_none
                    == Some(next_contracts::world_routine::WorldRoutineActivityV1::Rest)
                && relay.world_routine_record_revision_or_none == Some(1)
        }
        _ => false,
    };
    let duty_branch_report = scenario.world_services_tick_commits.first();
    let duty_branch_matches = duty_branch_report.is_some_and(|commit| {
        let report = &commit.runtime_report;
        report.tick == 0
            && report.interaction_availability.len() == 1
            && report.interaction_availability[0].code
                == next_contracts::world_routine::InteractionAvailabilityCodeV1::Available
            && report.mapping_receipts.len() == 1
            && report.mapping_receipts[0].frame_code
                == next_contracts::input::InputMappingCodeV1::Accepted
            && !report.mapping_receipts[0].derived_commands.is_empty()
    });
    let duty_journal_matches = duty_branch_report.is_some_and(|commit| {
        journal_projects_quest_state(
            &scenario,
            &commit.runtime_report.rpg_snapshot,
            "nextengine.reference-alpha.quest.active",
        )
        .unwrap_or(false)
    });
    let (rest_branch_matches, rest_journal_matches) = scenario
        .world_routine_rest_branch_or_none
        .as_ref()
        .map_or((false, false), |branch| {
            let commits = &branch.world_services_tick_commits;
            let branch_matches = if let [tick_0, tick_1, boundary, rest_interaction] =
                commits.as_slice()
            {
                let boundary_report = &boundary.runtime_report;
                let rest_report = &rest_interaction.runtime_report;
                tick_0.runtime_report.tick == 0
                    && tick_1.runtime_report.tick == 1
                    && boundary_report.tick == 2
                    && rest_report.tick == 3
                    && boundary_report
                        .command_batches
                        .iter()
                        .flat_map(|batch| batch.body.envelopes.iter())
                        .filter(|command| {
                            matches!(
                                command.payload,
                                next_contracts::command::CommandPayload::WorldRoutine(_)
                            )
                        })
                        .count()
                        == 1
                    && boundary_report
                        .events
                        .iter()
                        .filter(|event| {
                            matches!(
                                event.payload,
                                next_contracts::command::EventPayload::WorldRoutine(_)
                            )
                        })
                        .count()
                        == 1
                    && boundary.routine_snapshot_or_none.as_ref().is_some_and(|snapshot| {
                        snapshot.record.current_activity
                            == next_contracts::world_routine::WorldRoutineActivityV1::Rest
                            && snapshot.record.record_revision == 1
                    })
                    && branch.rest_query.code
                        == next_contracts::world_routine::InteractionAvailabilityCodeV1::WorldRoutineActivityUnavailable
                    && branch
                        .rest_query
                        .routine_binding_or_none
                        .as_ref()
                        .is_some_and(|binding| binding.routine_record_revision == 1)
                    && rest_report.interaction_availability
                        == vec![branch.rest_query.clone()]
                    && rest_report.mapping_receipts.len() == 1
                    && rest_report.mapping_receipts[0].frame_code
                        == next_contracts::input::InputMappingCodeV1::Accepted
                    && rest_report.mapping_receipts[0].derived_commands.is_empty()
                    && rest_report.rpg_snapshot == branch.initial_rpg_snapshot
                    && commits
                        .iter()
                        .all(|commit| commit.application_owner_segments.len() == 8)
            } else {
                false
            };
            let journal_matches = commits.last().is_some_and(|commit| {
                journal_projects_quest_state(
                    &scenario,
                    &commit.runtime_report.rpg_snapshot,
                    "nextengine.reference-alpha.quest.available",
                )
                .unwrap_or(false)
            });
            (branch_matches, journal_matches)
        });
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
            if relationship.source_id == scenario.quest_giver_character_id
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
    let report = PlayCheckReport {
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
        final_state_root: next_contracts::snapshot::world_checkpoint_with_cognition_v1_state_root(
            &checkpoint.runtime_snapshot,
            &checkpoint.rpg_snapshot,
            &checkpoint.physics_checkpoint,
            &scenario.world_streaming_snapshot,
            scenario.world_routine_snapshot_or_none.as_ref(),
            Some(&scenario.world_population_snapshot),
            &scenario.agent_cognition_snapshot,
            &scenario.agent_memory_snapshot,
        )?,
    };
    let command_archive_root = checkpoint
        .runtime_snapshot
        .command_ledger
        .body_archive
        .archive_root
        .to_hex();
    let command_identity_index_root = checkpoint
        .runtime_snapshot
        .command_ledger
        .identity_index
        .index_root
        .to_hex();
    if report.ticks != 32
        || report.final_pose.translation_micrometres != [0, 900_000, -200_000]
        || report.events != 46
        || report.rpg_events != 13
        || report.interactive_object_state.as_str()
            != next_contracts::rpg::CORE_INTERACTIVE_OBJECT_ACTIVATED_STATE_ID
        || report.dialogue_node_id.as_str() != "nextengine.reference-alpha.dialogue.completed"
        || report.quest_state_id.as_str() != "nextengine.reference-alpha.quest.completed"
        || report.npc_player_trust != 10
        || report.npc_health != 0
        || report.player_health != 50
        || report.world_streaming_generation != 2
        || command_archive_root
            != "403c49a6f27226edc8b1f568999b96083f30e3285f8bc0dac0672a276a8b2e76"
        || command_identity_index_root
            != "c20a1dd8d132994a6bdd38fc60153fb9cd5c5794f8a2e6a1a41c3976b6bac9d8"
        || report.final_command_ledger_hash.to_hex()
            != "0ab8988984c4ad53869e151bee887423cd60dece9ac94cdac1425ff800761651"
        || stage_checkpoint_count != 3
        || !stage_checkpoints_match_acceptance
        || !duty_branch_matches
        || !duty_journal_matches
        || !rest_branch_matches
        || !rest_journal_matches
    {
        return Err(PlayCheckError::AcceptanceMismatch(format!(
            "ticks={} pose={:?} events={} rpg_events={} object={} dialogue={} quest={} \
             trust={} npc_health={} player_health={} world_generation={} archive_root={} \
             identity_index_root={} ledger_root={} stage_checkpoints={} \
             stage_checkpoint_facts={:?} duty_branch={} duty_journal={} rest_branch={} rest_journal={}",
            report.ticks,
            report.final_pose.translation_micrometres,
            report.events,
            report.rpg_events,
            report.interactive_object_state,
            report.dialogue_node_id,
            report.quest_state_id,
            report.npc_player_trust,
            report.npc_health,
            report.player_health,
            report.world_streaming_generation,
            command_archive_root,
            command_identity_index_root,
            report.final_command_ledger_hash.to_hex(),
            stage_checkpoint_count,
            scenario.stage_checkpoints,
            duty_branch_matches,
            duty_journal_matches,
            rest_branch_matches,
            rest_journal_matches,
        )));
    }
    Ok(report)
}

fn journal_projects_quest_state(
    scenario: &ReferenceRunOutcomeV2,
    rpg: &RpgSnapshotV2,
    expected_state_id: &str,
) -> Result<bool, PlayCheckError> {
    let records = next_reference_game::read_only_screen_semantic_ui_records_for_ids(
        scenario.project_composition_lock_hash,
        scenario.player_character_id,
        scenario.quest_id,
        &[scenario.pickup_item_id, scenario.npc_weapon_item_id],
        &scenario.item_display_text_id,
        &scenario.quest_display_text_id,
        rpg,
    )?;
    Ok(records.iter().any(|record| {
        record
            .element
            .element_id
            .as_str()
            .starts_with("nextengine.ui.element.quest-journal.entry.")
            && record
                .element
                .text_or_none
                .as_ref()
                .and_then(|text| text.arguments.get(1))
                .is_some_and(|argument| {
                    matches!(
                        argument,
                        next_contracts::presentation::UiTextArgumentV1::TextId(state_id)
                            if state_id.as_str() == expected_state_id
                    )
                })
    }))
}

fn scratch_error(error: std::io::Error) -> PlayCheckError {
    PlayCheckError::Fixture(crate::NeutralFixtureError::Cleanup(error))
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
        PhysicsCollisionBackend::Reference => run_packaged_reference_fixture(
            "nextengine.physics-collision.reference",
            false,
            false,
            PhysicsLaunchOptions::default(),
        )?,
        PhysicsCollisionBackend::PhysX => run_packaged_reference_fixture(
            "nextengine.physics-collision.physx",
            false,
            true,
            PhysicsLaunchOptions::new(PhysicsBackendPolicy::RequirePhysX),
        )?,
        PhysicsCollisionBackend::Compare => {
            let reference = run_packaged_reference_fixture(
                "nextengine.physics-collision.compare",
                false,
                true,
                PhysicsLaunchOptions::new(PhysicsBackendPolicy::ReferenceOnly),
            )?;
            let physx = run_packaged_reference_fixture(
                "nextengine.physics-collision.compare",
                false,
                true,
                PhysicsLaunchOptions::new(PhysicsBackendPolicy::RequirePhysX),
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

fn run_packaged_reference_fixture(
    project_id: &str,
    include_interaction: bool,
    physx_compatible: bool,
    physics_options: PhysicsLaunchOptions,
) -> Result<ReferenceRunOutcomeV2, PlayCheckError> {
    let scratch = ScratchContext::new(&std::env::temp_dir()).map_err(scratch_error)?;
    let prepared = prepare_fixture_project_package_with_scratch(&scratch, project_id)?;
    let result = run_reference_game_with_backend(
        include_interaction,
        physx_compatible,
        physics_options,
        prepared.package.clone(),
    )
    .map_err(PlayCheckError::from);
    prepared.finish(result, scratch_error)
}
