use next_contracts::canonical::CanonicalDecodeLimits;
use next_contracts::persistence::ReplayManifestV6;

use crate::run_replay_manifest_v6_with_physics_options;

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
        &direct.reports,
        &direct.world_services_commits,
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
    let replay_bytes = replay_manifest
        .to_jcs_bytes()
        .map_err(|error| PersistenceReplayCheckError::new("encode replay V6", error.to_string()))?;
    let decoded_replay_manifest =
        ReplayManifestV6::from_jcs_bytes(&replay_bytes, CanonicalDecodeLimits::default()).map_err(
            |error| PersistenceReplayCheckError::new("decode replay V6", error.to_string()),
        )?;
    if decoded_replay_manifest != replay_manifest {
        return Err(PersistenceReplayCheckError::condition(
            "replay V6 JCS round trip is exact",
        ));
    }
    let replay = run_replay_manifest_v6_with_physics_options(
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
        &replay,
    )?;

    Ok(())
}
