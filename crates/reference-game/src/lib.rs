#![forbid(unsafe_code)]

mod camera;
mod dialogue;
mod error;
mod input;
mod live;
mod rpg;
mod runtime_bootstrap;
mod scenario;
mod session;
mod source;
mod topology;
mod ui;
mod ui_dialogue;

pub mod audio;

pub use dialogue::{ReferenceDialogueChoiceV1, ReferenceDialogueUiV1};
pub use error::{ReferenceGameError, ReferenceInputError};
pub use input::{
    ReferenceUiScreenV1, player_action_sample, player_equip_use_sample, player_interact_sample,
    player_melee_sample, player_pickup_sample,
};
pub use live::{
    PreparedReferenceGameAdvance, ReferenceGameDriverV1, ReferenceLiveDriverRecoveryV1,
    ReferenceLiveStateV1, ValidatedReferenceGameAdvance, reference_b0_presentation_profile_hash,
};
pub use rpg::{
    aggregate_payload, cooked_initial_interaction_outcome, cooked_interaction_outcome,
    cooked_project_rpg_snapshot, reference_aggregate,
};
pub use runtime_bootstrap::{ReferenceRuntimeBootstrap, build_reference_runtime_bootstrap};
pub use scenario::{
    ReferenceRunOutcomeV1, ReferenceStageCheckpointV1, run_reference_game,
    run_reference_game_with_backend,
};
pub use session::{
    ReferenceGameSession, build_reference_game_session, build_reference_game_session_with_profile,
};
pub use source::{
    REFERENCE_GAME_PROJECT_ID, project_source_v2, project_source_v2_with_id,
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
