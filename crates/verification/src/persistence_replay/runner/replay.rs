use next_contracts::canonical::CanonicalDecodeLimits;
use next_contracts::ids::{SchemaId, StateRoot};
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
    verify_public_replay_inspector(
        &decoded_replay_manifest,
        &direct._project_package.path().join("r6f-replay-v10.jcs"),
        direct._project_package.path(),
    )?;
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

fn verify_public_replay_inspector(
    replay: &ReplayManifestV10,
    replay_path: &std::path::Path,
    content_store: &std::path::Path,
) -> Result<(), PersistenceReplayCheckError> {
    let bytes = replay.to_jcs_bytes().map_err(|error| {
        PersistenceReplayCheckError::new("encode public replay fixture", error.to_string())
    })?;
    std::fs::write(replay_path, &bytes).map_err(|error| {
        PersistenceReplayCheckError::new("write public replay fixture", error.to_string())
    })?;
    let replay_argument = replay_path.as_os_str().to_owned();
    let store_argument = content_store.as_os_str().to_owned();
    let validate = next_cli::execute([
        "replay".into(),
        "validate".into(),
        "--replay".into(),
        replay_argument.clone(),
        "--content-store".into(),
        store_argument.clone(),
    ]);
    let next_cli::CreatorCliReportV1::Replay(next_cli::CreatorReplayCommandReportV1::Pass(
        validate,
    )) = validate
    else {
        return Err(PersistenceReplayCheckError::condition(
            "public replay validate accepts the generated exact-project V10 fixture",
        ));
    };
    let next_cli::CreatorReplayDetailsV1::Validate {
        replay: identity,
        validation_state,
        ..
    } = validate.details
    else {
        return Err(PersistenceReplayCheckError::condition(
            "public replay validate returns validate details",
        ));
    };
    let first_tick = replay
        .ticks
        .first()
        .ok_or_else(|| PersistenceReplayCheckError::condition("public replay has a first tick"))?
        .tick;
    if validation_state != "validated-not-run"
        || identity.tick_count
            != u32::try_from(replay.ticks.len()).map_err(|error| {
                PersistenceReplayCheckError::new("public replay tick count", error.to_string())
            })?
        || identity.project_id != replay.compatibility.project_id.as_str()
    {
        return Err(PersistenceReplayCheckError::condition(
            "public replay validate reports the exact bounded identity",
        ));
    }

    let missing_tick = first_tick
        .checked_add(u64::try_from(replay.ticks.len()).map_err(|error| {
            PersistenceReplayCheckError::new("missing replay tick", error.to_string())
        })?)
        .ok_or_else(|| PersistenceReplayCheckError::condition("missing replay tick is bounded"))?;
    let missing = next_cli::execute([
        "replay".into(),
        "inspect".into(),
        "--replay".into(),
        replay_argument.clone(),
        "--content-store".into(),
        store_argument.clone(),
        "--tick".into(),
        missing_tick.to_string().into(),
        "--domain".into(),
        "runtime".into(),
    ]);
    let next_cli::CreatorCliReportV1::Replay(next_cli::CreatorReplayCommandReportV1::Fail(missing)) =
        missing
    else {
        return Err(PersistenceReplayCheckError::condition(
            "public replay inspect rejects an absent tick",
        ));
    };
    if missing.diagnostic.code != "CREATOR_REPLAY_TICK_NOT_FOUND" {
        return Err(PersistenceReplayCheckError::condition(
            "public replay absent tick has a stable diagnostic",
        ));
    }

    let retired_path = content_store.join("r6f-replay-retired-v9.jcs");
    let retired = String::from_utf8(bytes.clone())
        .map_err(|error| {
            PersistenceReplayCheckError::new("decode replay fixture text", error.to_string())
        })?
        .replacen("\"schema_version\":10", "\"schema_version\":9", 1);
    if retired.as_bytes() == bytes {
        return Err(PersistenceReplayCheckError::condition(
            "retired replay fixture changes the outer version",
        ));
    }
    std::fs::write(&retired_path, retired).map_err(|error| {
        PersistenceReplayCheckError::new("write retired replay", error.to_string())
    })?;
    let retired = next_cli::execute([
        "replay".into(),
        "validate".into(),
        "--replay".into(),
        retired_path.into_os_string(),
        "--content-store".into(),
        store_argument.clone(),
    ]);
    let next_cli::CreatorCliReportV1::Replay(next_cli::CreatorReplayCommandReportV1::Fail(retired)) =
        retired
    else {
        return Err(PersistenceReplayCheckError::condition(
            "public replay validate rejects retired V9",
        ));
    };
    if retired.diagnostic.code != "UNSUPPORTED_REPLAY_MANIFEST_VERSION" {
        return Err(PersistenceReplayCheckError::condition(
            "retired replay fails at the outer version boundary",
        ));
    }

    for domain in ["runtime", "world-services", "physics", "owners"] {
        let inspect = next_cli::execute([
            "replay".into(),
            "inspect".into(),
            "--replay".into(),
            replay_argument.clone(),
            "--content-store".into(),
            store_argument.clone(),
            "--tick".into(),
            first_tick.to_string().into(),
            "--domain".into(),
            domain.into(),
        ]);
        let next_cli::CreatorCliReportV1::Replay(next_cli::CreatorReplayCommandReportV1::Pass(
            inspect,
        )) = inspect
        else {
            return Err(PersistenceReplayCheckError::condition(
                "public replay inspect verifies every bounded domain",
            ));
        };
        let next_cli::CreatorReplayDetailsV1::Inspect {
            verification_state,
            projection,
            ..
        } = inspect.details
        else {
            return Err(PersistenceReplayCheckError::condition(
                "public replay inspect returns inspect details",
            ));
        };
        let projection_tick = match (domain, projection) {
            ("runtime", next_cli::CreatorReplayDomainProjectionV1::Runtime { tick, .. })
            | (
                "world-services",
                next_cli::CreatorReplayDomainProjectionV1::WorldServices { tick, .. },
            )
            | ("physics", next_cli::CreatorReplayDomainProjectionV1::Physics { tick, .. })
            | ("owners", next_cli::CreatorReplayDomainProjectionV1::Owners { tick, .. }) => tick,
            _ => {
                return Err(PersistenceReplayCheckError::condition(
                    "public replay inspect returns the requested domain projection",
                ));
            }
        };
        if verification_state != "replayed-exact" || projection_tick != first_tick {
            return Err(PersistenceReplayCheckError::condition(
                "public replay domain projection follows exact replay verification",
            ));
        }
    }

    let actual_first_state_root = replay
        .compare_points
        .first()
        .ok_or_else(|| PersistenceReplayCheckError::condition("replay has a compare point"))?
        .state_root;
    let divergent_state_root = StateRoot::from_bytes([0x6f; 32]);
    let mut divergent = replay.clone();
    divergent
        .compare_points
        .first_mut()
        .ok_or_else(|| {
            PersistenceReplayCheckError::condition("divergent replay has compare point")
        })?
        .state_root = divergent_state_root;
    let divergent_path = content_store.join("r6f-replay-divergent-v10.jcs");
    std::fs::write(
        &divergent_path,
        divergent.to_jcs_bytes().map_err(|error| {
            PersistenceReplayCheckError::new("encode divergent replay", error.to_string())
        })?,
    )
    .map_err(|error| {
        PersistenceReplayCheckError::new("write divergent replay", error.to_string())
    })?;
    let divergence = next_cli::execute([
        "replay".into(),
        "inspect".into(),
        "--replay".into(),
        divergent_path.into_os_string(),
        "--content-store".into(),
        store_argument.clone(),
        "--tick".into(),
        first_tick.to_string().into(),
        "--domain".into(),
        "runtime".into(),
    ]);
    let next_cli::CreatorCliReportV1::Replay(next_cli::CreatorReplayCommandReportV1::Fail(
        divergence,
    )) = divergence
    else {
        return Err(PersistenceReplayCheckError::condition(
            "public replay inspect rejects a deterministic divergence",
        ));
    };
    let Some(first) = divergence.diagnostic.divergence else {
        return Err(PersistenceReplayCheckError::condition(
            "public replay divergence identifies the first cause",
        ));
    };
    if divergence.diagnostic.code != "NONDETERMINISTIC_RESULT"
        || first.first_divergent_tick != first_tick
        || first.stage != "application-state-root"
        || first.owner != "application"
        || first.expected.as_deref() != Some(divergent_state_root.to_hex().as_str())
        || first.actual.as_deref() != Some(actual_first_state_root.to_hex().as_str())
    {
        return Err(PersistenceReplayCheckError::condition(
            "public replay divergence reports first tick, stage and owner",
        ));
    }

    let mut mismatched = replay.clone();
    mismatched.compatibility.project_id =
        SchemaId::new("nextengine.mismatched-project").map_err(|error| {
            PersistenceReplayCheckError::new("mismatched replay project ID", error.to_string())
        })?;
    let mismatch_path = content_store.join("r6f-replay-project-mismatch-v10.jcs");
    std::fs::write(
        &mismatch_path,
        mismatched.to_jcs_bytes().map_err(|error| {
            PersistenceReplayCheckError::new("encode mismatched replay", error.to_string())
        })?,
    )
    .map_err(|error| {
        PersistenceReplayCheckError::new("write mismatched replay", error.to_string())
    })?;
    let mismatch = next_cli::execute([
        "replay".into(),
        "validate".into(),
        "--replay".into(),
        mismatch_path.into_os_string(),
        "--content-store".into(),
        store_argument,
    ]);
    let next_cli::CreatorCliReportV1::Replay(next_cli::CreatorReplayCommandReportV1::Fail(
        mismatch,
    )) = mismatch
    else {
        return Err(PersistenceReplayCheckError::condition(
            "public replay validate rejects a mismatched project",
        ));
    };
    if mismatch.diagnostic.code != "CREATOR_REPLAY_PROJECT_MISMATCH" {
        return Err(PersistenceReplayCheckError::condition(
            "public replay mismatch has the stable project diagnostic",
        ));
    }
    Ok(())
}
