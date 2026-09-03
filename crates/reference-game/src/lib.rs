#![forbid(unsafe_code)]

mod body_projection;
mod camera;
mod dialogue;
mod error;
mod input;
mod live;
mod physical_animation;
mod rpg;
mod runtime_bootstrap;
mod scenario;
mod session;
mod source;
mod topology;
mod ui;
mod ui_dialogue;
mod water;
mod water_presentation;

pub mod audio;

pub use body_projection::{ReferenceBodyProjectionSetV1, ReferenceBodyProjectionV1};
pub use dialogue::{ReferenceDialogueChoiceV1, ReferenceDialogueUiV1};
pub use error::{ReferenceGameError, ReferenceInputError};
pub use input::{
    ReferenceUiScreenV1, player_action_sample, player_equip_use_sample, player_interact_sample,
    player_melee_sample, player_pickup_sample,
};
pub use live::{
    PreparedReferenceGameAdvance, REFERENCE_BULK_TIME_MAX_TICKS_V1, ReferenceBulkTimeAdvanceV1,
    ReferenceBulkTimeStopReasonV1, ReferenceGameDriverV2, ReferenceLiveDriverRecoveryV1,
    ReferenceLiveStateV2, ValidatedReferenceGameAdvance, reference_b0_presentation_profile_hash,
};
pub use physical_animation::{
    reference_physical_animation_owner, restore_reference_physical_animation_owner,
};
pub use rpg::{
    aggregate_payload, cooked_initial_interaction_outcome, cooked_interaction_outcome,
    cooked_project_rpg_snapshot, reference_aggregate,
};
pub use runtime_bootstrap::{ReferenceRuntimeBootstrap, build_reference_runtime_bootstrap};
pub use scenario::{
    ReferenceRunOutcomeV2, ReferenceStageCheckpointV2, ReferenceWorldRoutineRestBranchV1,
    reference_character_skinning_record_from_lod_projection, run_reference_game,
    run_reference_game_with_backend,
};
pub use session::{
    REFERENCE_SPAWN_TRANSLATION_MICROMETRES, ReferenceCapsuleCourseV1, ReferenceGameSession,
    ReferenceSpawnOverrideV1, build_reference_game_session,
    build_reference_game_session_with_options, build_reference_game_session_with_profile,
};
pub use source::{
    REFERENCE_GAME_PROJECT_ID, project_source_v7, project_source_v7_with_id,
    reference_alpha_project_directory,
};
pub use topology::{ReferenceChunkRouteEntryV1, ReferenceWorldTopologyV1};
pub use ui::{
    HUD_ACTION_ACCEPT_TEXT_ID, HUD_ACTION_COMBAT_TEXT_ID, HUD_ACTION_COMPLETE_TEXT_ID,
    HUD_ACTION_ELEMENT_ID, HUD_ACTION_EQUIP_TEXT_ID, HUD_ACTION_PICKUP_TEXT_ID,
    HUD_ACTION_RELAY_TEXT_ID, HUD_ACTION_RETURN_TEXT_ID, PAUSE_MENU_LOAD_ELEMENT_ID,
    PAUSE_MENU_LOADED_TEXT_ID, PAUSE_MENU_RESUME_ELEMENT_ID, PAUSE_MENU_SAVE_ELEMENT_ID,
    PAUSE_MENU_SAVED_TEXT_ID, PAUSE_MENU_SURFACE_ID, hud_semantic_ui_records_for_ids,
    pause_menu_semantic_ui_records, read_only_screen_semantic_ui_records_for_ids,
};
pub use ui_dialogue::dialogue_semantic_ui_records_for_ids;
pub use water::{
    REFERENCE_WATER_BASIN_ID, REFERENCE_WATER_BASIN_INITIAL_LEVEL_MICROMETRES,
    REFERENCE_WATER_BASIN_MAXIMUM_MICROMETRES, REFERENCE_WATER_BASIN_MINIMUM_MICROMETRES,
    REFERENCE_WATER_BASIN_RIM_BODY_ID, REFERENCE_WATER_BASIN_RIM_BOXES_MICROMETRES,
    REFERENCE_WATER_BASIN_SWIMMING_DEPTH_MICROMETRES, REFERENCE_WATER_CRATE_BODY_ID,
    REFERENCE_WATER_CRATE_HALF_EXTENTS_MICROMETRES,
    REFERENCE_WATER_CRATE_INITIAL_TRANSLATION_MICROMETRES,
    REFERENCE_WATER_CRATE_MASS_MICROKILOGRAMS, REFERENCE_WATER_FLOW_GATE_ID,
    REFERENCE_WATER_FLOW_SINK_ID, REFERENCE_WATER_FLOW_SOURCE_ID,
    REFERENCE_WATER_FLOW_TICKS_PER_SECOND, REFERENCE_WATER_VESSEL_A_ID,
    REFERENCE_WATER_VESSEL_A_SURFACE_OBJECT_ID, REFERENCE_WATER_VESSEL_B_ID,
    REFERENCE_WATER_VESSEL_B_SURFACE_OBJECT_ID, player_submersion,
    reference_water_basin_definition, reference_water_flow, reference_water_flow_edges,
    reference_water_vessel_definitions, reference_water_volumes, volume_surface_translation,
    water_surface_translation,
};
pub use water_presentation::{
    WATER_FALL_DROP_MICROMETRES, WATER_JET_MAX_PARTICLES, WATER_JET_RADIUS_MICROMETRES,
    WATER_PRESENTATION_FRAMES_PER_SECOND, WATER_RIPPLE_CAP_MICROMETRES,
    WATER_SURFACE_INDEX_CAPACITY, WATER_SURFACE_VERTEX_CAPACITY, WaterEdgePresentationKindV1,
    WaterEdgePresentationV1, WaterJetParticlesV1, WaterPresentationFrameV1, WaterSurfaceBindingV1,
    WaterSurfaceUpdateV1, compute_water_presentation_frame, floating_boxes,
    reference_water_surface_bindings,
};
