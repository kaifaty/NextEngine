use next_contracts::canonical::CanonicalDecodeLimits;
use next_contracts::persistence::ReplayManifestV10;

use crate::run_replay_manifest_v10_with_physics_options;

use super::super::PersistenceReplayCheckError;
use super::super::replay_support::{compare_replay, replay_manifest};
use super::{AgentEvidence, DirectScenario, RestoredScenario};

pub(super) fn verify(
    direct: &DirectScenario,
    restored: &RestoredScenario,
    agent: &AgentEvidence,
) -> Result<(), PersistenceReplayCheckError> {
    let replay_manifest = replay_manifest(
        restored.compatibility.clone(),
        &direct.fixture.authority,
        &direct.initial_checkpoint,
        &direct.initial_world_snapshot,
        direct.initial_routine_snapshot_or_none.as_ref(),
        &direct.initial_population_snapshot,
        &direct.initial_activity_snapshot,
        &direct.initial_agent_snapshot,
        &direct.initial_memory_snapshot,
        &direct.initial_physical_animation_snapshot,
        &direct.reports,
        &direct.world_services_commits,
        &direct.physical_animation_snapshots,
        &direct.replay_streaming_inputs,
        &direct.replay_direct_commands,
    )?;
    if !direct
        .replay_direct_commands
        .iter()
        .flatten()
        .any(|command| command == &agent.replay_command)
    {
        return Err(PersistenceReplayCheckError::condition(
            "recorded replay contains the exact planned agent command",
        ));
    }
    if !direct
        .replay_direct_commands
        .iter()
        .flatten()
        .any(|command| {
            matches!(
                &command.payload,
                next_contracts::command::CommandPayload::RootMotion(_)
            )
        })
    {
        return Err(PersistenceReplayCheckError::condition(
            "Replay V10 records the complete root-motion proposal command",
        ));
    }
    let replay_bytes = replay_manifest.to_jcs_bytes().map_err(|error| {
        PersistenceReplayCheckError::new("encode replay V10", error.to_string())
    })?;
    let decoded_replay_manifest =
        ReplayManifestV10::from_jcs_bytes(&replay_bytes, CanonicalDecodeLimits::default())
            .map_err(|error| {
                PersistenceReplayCheckError::new("decode replay V10", error.to_string())
            })?;
    if decoded_replay_manifest != replay_manifest {
        return Err(PersistenceReplayCheckError::condition(
            "replay V10 JCS round trip is exact",
        ));
    }
    let mut missing_initial_animation = decoded_replay_manifest.clone();
    missing_initial_animation
        .initial_owner_segments
        .retain(|segment| {
            segment.descriptor.owner_id.as_str()
                != next_contracts::physical_animation::PHYSICAL_ANIMATION_SNAPSHOT_OWNER_ID
        });
    if missing_initial_animation
        .validate_and_decode(CanonicalDecodeLimits::default())
        .is_ok()
    {
        return Err(PersistenceReplayCheckError::condition(
            "replay V10 rejects a missing initial physical-animation owner",
        ));
    }
    let mut missing_compare_animation = decoded_replay_manifest.clone();
    let first_compare_point = missing_compare_animation
        .compare_points
        .first_mut()
        .ok_or_else(|| PersistenceReplayCheckError::condition("replay V10 has compare points"))?;
    first_compare_point.owner_segments.retain(|segment| {
        segment.owner_id.as_str()
            != next_contracts::physical_animation::PHYSICAL_ANIMATION_SNAPSHOT_OWNER_ID
    });
    if missing_compare_animation
        .validate_and_decode(CanonicalDecodeLimits::default())
        .is_ok()
    {
        return Err(PersistenceReplayCheckError::condition(
            "replay V10 rejects a missing compare-point physical-animation owner",
        ));
    }
    let replay = run_replay_manifest_v10_with_physics_options(
        &decoded_replay_manifest,
        next_project::ActivatedProjectPackage {
            project: direct.fixture.activated_project.clone(),
            content_generation: direct.content_generation.clone(),
        },
        direct.physics_options,
    )
    .map_err(|error| PersistenceReplayCheckError::new("closed-batch replay", error.to_string()))?;
    compare_replay(
        &direct.runtime,
        &direct.reports,
        &direct.world_services_commits,
        &direct.physical_animation_snapshots,
        &replay,
    )?;

    Ok(())
}
