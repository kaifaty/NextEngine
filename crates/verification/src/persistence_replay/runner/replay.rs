use next_contracts::canonical::CanonicalDecodeLimits;
use next_contracts::persistence::ReplayManifestV5;
use next_world::WorldStreamerV1;

use crate::run_replay_manifest_v5_with_definitions_and_physics_options;

use super::super::PersistenceReplayCheckError;
use super::super::replay_support::{compare_replay, replay_manifest, transition_world};
use super::{AgentEvidence, DirectScenario, RestoredScenario};

pub(super) fn verify(
    direct: &DirectScenario,
    restored: &RestoredScenario,
    agent: &AgentEvidence,
) -> Result<(), PersistenceReplayCheckError> {
    let replay_manifest = replay_manifest(
        restored.compatibility.clone(),
        &direct.fixture.authority,
        direct.initial_checkpoint.clone(),
        &direct.reports,
        vec![
            direct.direct_commands.clone(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![agent.replay_command.clone()],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ],
    )?;
    let replay_bytes = replay_manifest
        .to_jcs_bytes()
        .map_err(|error| PersistenceReplayCheckError::new("encode replay V5", error.to_string()))?;
    let decoded_replay_manifest =
        ReplayManifestV5::from_jcs_bytes(&replay_bytes, CanonicalDecodeLimits::default()).map_err(
            |error| PersistenceReplayCheckError::new("decode replay V5", error.to_string()),
        )?;
    if decoded_replay_manifest != replay_manifest {
        return Err(PersistenceReplayCheckError::condition(
            "replay V5 JCS round trip is exact",
        ));
    }
    let replay = run_replay_manifest_v5_with_definitions_and_physics_options(
        &decoded_replay_manifest,
        direct.fixture.activated_project.rpg_definitions.clone(),
        direct.physics_options,
    )
    .map_err(|error| PersistenceReplayCheckError::new("closed-batch replay", error.to_string()))?;
    compare_replay(&direct.runtime, &direct.reports, &replay)?;

    let mut replay_world = WorldStreamerV1::activate(
        direct.fixture.activated_project.clone(),
        direct.content_generation.clone(),
        direct.initial_chunk_id.clone(),
    )
    .map_err(|error| {
        PersistenceReplayCheckError::new("activate replay world", error.to_string())
    })?;
    transition_world(
        &mut replay_world,
        direct.transition_chunk_id.clone(),
        11,
        "replay forward world",
    )?;
    transition_world(
        &mut replay_world,
        direct.initial_chunk_id.clone(),
        16,
        "replay return world",
    )?;
    if replay_world.snapshot() != direct.world.snapshot() {
        return Err(PersistenceReplayCheckError::condition(
            "world streaming replay reaches the same state",
        ));
    }

    Ok(())
}
