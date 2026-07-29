mod checks;
mod construction;
mod error;
mod input;
mod rpg;
mod scenario;

#[cfg(test)]
mod tests;

pub use checks::{
    GameCheckReport, PhysicsCollisionBackend, PhysicsCollisionCheckReport, PlayCheckReport,
    PreparedGameFrameV1, prepare_game_frame, prepare_game_frame_with_activated_project,
    run_game_check, run_physics_collision_check, run_physics_collision_check_with_backend,
    run_play_check, run_play_check_with_activated_project,
};
pub use construction::{
    NeutralPlayerFixture, build_neutral_player_fixture,
    build_neutral_player_fixture_from_activated_project, build_physx_player_fixture,
};
pub use error::{CanonicalFixtureError, PlayCheckError};
pub use input::{
    player_action_sample, player_equip_use_sample, player_interact_sample, player_melee_sample,
    player_pickup_sample,
};
pub use rpg::{cooked_interaction_outcome, cooked_project_rpg_snapshot};

pub(crate) use rpg::fixture_aggregate;
