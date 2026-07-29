#![forbid(unsafe_code)]

mod error;
mod input;
mod rpg;
mod runtime_bootstrap;
mod scenario;
mod session;
mod source;

pub use error::{ReferenceGameError, ReferenceInputError};
pub use input::{
    player_action_sample, player_equip_use_sample, player_interact_sample, player_melee_sample,
    player_pickup_sample,
};
pub use rpg::{
    aggregate_payload, cooked_interaction_outcome, cooked_project_rpg_snapshot, reference_aggregate,
};
pub use runtime_bootstrap::{ReferenceRuntimeBootstrap, build_reference_runtime_bootstrap};
pub use scenario::{ReferenceRunOutcomeV1, run_reference_game, run_reference_game_with_backend};
pub use session::{
    ReferenceGameSession, build_reference_game_session, build_reference_game_session_with_profile,
};
pub use source::{REFERENCE_GAME_PROJECT_ID, project_source_v2, project_source_v2_with_id};
