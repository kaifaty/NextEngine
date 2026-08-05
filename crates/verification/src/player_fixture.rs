mod checks;
mod error;

pub mod audio_check;

#[cfg(test)]
mod tests;

use next_assets::ContentStore;
use next_contracts::project::ActivatedProjectV2;

pub use checks::{
    GameCheckReport, PhysicsCollisionBackend, PhysicsCollisionCheckReport, PlayCheckReport,
    PreparedGameFrameV1, prepare_game_frame, prepare_game_frame_in,
    prepare_game_frame_with_activated_project, run_game_check, run_physics_collision_check,
    run_physics_collision_check_with_backend, run_play_check, run_play_check_in,
    run_play_check_with_activated_project,
};
pub(crate) use checks::{prepare_game_frame_with_scratch, run_play_check_with_scratch};
pub use error::PlayCheckError;
pub use next_reference_game::{
    ReferenceGameSession as NeutralPlayerFixture, ReferenceInputError as CanonicalFixtureError,
    cooked_interaction_outcome, cooked_project_rpg_snapshot, player_action_sample,
    player_equip_use_sample, player_interact_sample, player_melee_sample, player_pickup_sample,
    reference_aggregate as fixture_aggregate,
};

use crate::NeutralFixtureError;
use crate::scratch::ScratchContext;

pub fn build_neutral_player_fixture(
    project_id: &str,
) -> Result<NeutralPlayerFixture, NeutralFixtureError> {
    let activated = activate_fixture_project(project_id)?;
    Ok(next_reference_game::build_reference_game_session(
        activated,
    )?)
}

pub fn build_physx_player_fixture(
    project_id: &str,
) -> Result<NeutralPlayerFixture, NeutralFixtureError> {
    let activated = activate_fixture_project(project_id)?;
    Ok(next_reference_game::build_reference_game_session_with_profile(activated, true)?)
}

pub(crate) fn build_neutral_player_fixture_with_scratch(
    scratch: &ScratchContext,
    project_id: &str,
) -> Result<NeutralPlayerFixture, NeutralFixtureError> {
    let activated = activate_fixture_project_with_scratch(scratch, project_id)?;
    Ok(next_reference_game::build_reference_game_session(
        activated,
    )?)
}

pub(crate) fn build_physx_player_fixture_with_scratch(
    scratch: &ScratchContext,
    project_id: &str,
) -> Result<NeutralPlayerFixture, NeutralFixtureError> {
    let activated = activate_fixture_project_with_scratch(scratch, project_id)?;
    Ok(next_reference_game::build_reference_game_session_with_profile(activated, true)?)
}

pub fn build_neutral_player_fixture_from_activated_project(
    activated_project: ActivatedProjectV2,
) -> Result<NeutralPlayerFixture, NeutralFixtureError> {
    Ok(next_reference_game::build_reference_game_session(
        activated_project,
    )?)
}

pub(crate) fn activate_fixture_project(
    project_id: &str,
) -> Result<ActivatedProjectV2, NeutralFixtureError> {
    let scratch =
        ScratchContext::new(&std::env::temp_dir()).map_err(NeutralFixtureError::Cleanup)?;
    activate_fixture_project_with_scratch(&scratch, project_id)
}

pub(crate) fn activate_fixture_project_with_scratch(
    scratch: &ScratchContext,
    project_id: &str,
) -> Result<ActivatedProjectV2, NeutralFixtureError> {
    let source = next_reference_game::project_source_v2_with_id(project_id)?;
    let cooked = next_project::cook_project_v1(source)?;
    let directory = scratch
        .create_directory("play-project")
        .map_err(NeutralFixtureError::Cleanup)?;
    let store = ContentStore::new(directory.path());
    let result = (|| {
        store.publish(&cooked.publication()?)?;
        Ok(next_project::activate_project(&store)?)
    })();
    directory.finish(result, NeutralFixtureError::Cleanup)
}
