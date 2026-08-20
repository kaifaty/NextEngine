use next_contracts::canonical::{CanonicalDecodeLimits, CanonicalError};
use next_contracts::ids::{CommandLedgerHash, SchemaId, StateRoot};
use next_contracts::persistence::{
    ManifestValidationError, ReplayComparePointV9, ReplayManifestV10, SaveCompatibility,
    SaveSegmentDescriptor, replay_physics_query_batch_hash, replay_physics_query_results_hash,
    replay_targeting_query_trace_hash,
};
use next_contracts::snapshot::WorldCheckpointV4;
use next_runtime::RuntimeReplayError;

use super::{ReplayDivergenceV1, ReplayError};

impl ReplayError {
    #[must_use]
    pub fn first_divergence(&self) -> Option<ReplayDivergenceV1> {
        match self {
            Self::ComparePointMismatch(mismatch) => Some(ReplayDivergenceV1 {
                first_divergent_tick: mismatch.first_divergent_tick,
                stage: mismatch.stage,
                owner: mismatch.owner,
                expected: match mismatch.stage {
                    "application-state-root" => Some(mismatch.expected_state_root.to_hex()),
                    "command-ledger" => Some(mismatch.expected_command_ledger_hash.to_hex()),
                    _ => None,
                },
                actual: match mismatch.stage {
                    "application-state-root" => Some(mismatch.actual_state_root.to_hex()),
                    "command-ledger" => Some(mismatch.actual_command_ledger_hash.to_hex()),
                    _ => None,
                },
            }),
            Self::RecordedStageMismatch { tick, stage } => Some(ReplayDivergenceV1 {
                first_divergent_tick: *tick,
                stage,
                owner: replay_stage_owner(stage),
                expected: None,
                actual: None,
            }),
            Self::NondeterministicResult {
                first_divergent_tick,
                expected_state_root,
                actual_state_root,
            } => Some(ReplayDivergenceV1 {
                first_divergent_tick: *first_divergent_tick,
                stage: "application-state-root",
                owner: "application",
                expected: expected_state_root.map(StateRoot::to_hex),
                actual: actual_state_root.map(StateRoot::to_hex),
            }),
            _ => None,
        }
    }
}

pub fn validate_replay_manifest_v10_for_project(
    manifest: &ReplayManifestV10,
    project: &next_contracts::project::ActivatedProjectV8,
) -> Result<(), ReplayError> {
    let (initial, _) = manifest.validate_and_decode(CanonicalDecodeLimits::default())?;
    validate_replay_project_compatibility(&manifest.compatibility, project, &initial.checkpoint)
}

pub fn replay_compatibility_for_project(
    project: &next_contracts::project::ActivatedProjectV8,
    checkpoint: &WorldCheckpointV4,
) -> Result<SaveCompatibility, ReplayError> {
    let profile = checkpoint.runtime_snapshot.tick_rate_profile;
    Ok(SaveCompatibility {
        engine_build_hash: project.project_lock.runtime_determinism_profile_sha256,
        game_build_hash: project.project_lock.authoring_sha256,
        project_id: SchemaId::new(project.project_lock.project_id.as_str())
            .map_err(CanonicalError::InvalidIdentifier)?,
        schema_registry_hash: project.project_lock.schema_registry_manifest_sha256,
        content_manifest_hash: project.project_lock.content_manifest_sha256,
        mechanics_lock_hash: project.project_lock.mechanics_lock_sha256,
        tick_settings: next_contracts::persistence::TickSettings {
            gameplay_hz: profile.gameplay_hz,
            physics_hz: profile.physics_hz(),
            motor_hz: profile.physics_hz() / profile.motor_period_physics_substeps,
        },
        loaded_chunk_revisions: Vec::new(),
        rng_stream_states: Vec::new(),
        physical_bindings: Vec::new(),
        policy_state_schemas: Vec::new(),
        plugin_script_bindings: Vec::new(),
    })
}

pub(super) fn validate_replay_project_compatibility(
    compatibility: &SaveCompatibility,
    project: &next_contracts::project::ActivatedProjectV8,
    checkpoint: &WorldCheckpointV4,
) -> Result<(), ReplayError> {
    let expected = replay_compatibility_for_project(project, checkpoint)?;
    if compatibility.engine_build_hash != expected.engine_build_hash
        || compatibility.game_build_hash != expected.game_build_hash
        || compatibility.project_id != expected.project_id
        || compatibility.schema_registry_hash != expected.schema_registry_hash
        || compatibility.content_manifest_hash != expected.content_manifest_hash
        || compatibility.mechanics_lock_hash != expected.mechanics_lock_hash
        || compatibility.tick_settings != expected.tick_settings
    {
        return Err(ManifestValidationError::CompatibilityMismatch.into());
    }
    Ok(())
}

pub(super) fn first_compare_point_divergence(
    state_root: StateRoot,
    command_ledger_hash: CommandLedgerHash,
    owner_segments: &[SaveSegmentDescriptor],
    report: &next_runtime::TickReport,
    compare_point: &ReplayComparePointV9,
) -> Result<(&'static str, &'static str), ReplayError> {
    let result = if state_root != compare_point.state_root {
        ("application-state-root", "application")
    } else if command_ledger_hash != compare_point.command_ledger_hash {
        ("command-ledger", "runtime")
    } else if owner_segments != compare_point.owner_segments {
        ("owner-segments", "application")
    } else if report.closed_ingress_batch.batch_hash != compare_point.closed_ingress_batch_hash {
        ("closed-ingress-batch", "runtime")
    } else if report.command_batches[0].batch_hash != compare_point.ingress_command_batch_hash {
        ("ingress-command-batch", "runtime")
    } else if report.physics_step_input.input_hash()? != compare_point.physics_step_input_hash {
        ("physics-step-input", "physics")
    } else if report.contact_batch.batch_hash != compare_point.contact_batch_hash {
        ("physics-contact-batch", "physics")
    } else if replay_physics_query_batch_hash(&report.physics_query_batch)?
        != compare_point.physics_query_batch_hash
    {
        ("physics-query-batch", "physics")
    } else if replay_physics_query_results_hash(&report.physics_query_results)?
        != compare_point.physics_query_results_hash
    {
        ("physics-query-results", "physics")
    } else if replay_targeting_query_trace_hash(
        &report.targeting_intents,
        &report.authoritative_targeting_queries,
    )? != compare_point.targeting_query_trace_hash
    {
        ("targeting-query-trace", "player-targeting")
    } else if report.command_batches[1].batch_hash != compare_point.outcome_command_batch_hash {
        ("outcome-command-batch", "runtime")
    } else if next_contracts::world_routine::interaction_availability_batch_hash(
        &report.interaction_availability,
    )
    .map_err(ManifestValidationError::from)?
        != compare_point.interaction_availability_hash
    {
        ("interaction-availability", "world-routine")
    } else {
        ("unknown-compare-point", "application")
    };
    Ok(result)
}

pub(super) const fn replay_stage_owner(stage: &str) -> &'static str {
    match stage.as_bytes() {
        b"world-streaming-input" => "world-streaming",
        b"physical-animation-owner" => "physical-animation",
        b"physics-step-input"
        | b"physics-contact-batch"
        | b"physics-query-batch"
        | b"physics-query-results" => "physics",
        b"targeting-intents" | b"targeting-queries" => "player-targeting",
        b"interaction-availability" => "world-routine",
        b"closed-ingress-command-outcome"
        | b"replay-driver"
        | b"replay-staging-restore"
        | b"ingress-command-batch"
        | b"outcome-command-batch"
        | b"runtime" => "runtime",
        b"closed-authoritative-query-outcome" => "authoritative-query",
        _ => "application",
    }
}

pub(super) fn runtime_replay_divergence(
    error: RuntimeReplayError,
    fallback_tick: u64,
) -> ReplayError {
    let (tick, stage) = match error {
        RuntimeReplayError::Runtime(_) => (fallback_tick, "runtime"),
        RuntimeReplayError::Restore(_) => (fallback_tick, "replay-staging-restore"),
        RuntimeReplayError::CommandBatchMismatch { tick, phase } => (
            tick,
            match phase {
                next_contracts::command::CommandPhase::Ingress => "ingress-command-batch",
                next_contracts::command::CommandPhase::Outcome => "outcome-command-batch",
            },
        ),
        RuntimeReplayError::PhysicsStepInputMismatch { tick } => (tick, "physics-step-input"),
        RuntimeReplayError::ContactBatchMismatch { tick } => (tick, "physics-contact-batch"),
        RuntimeReplayError::TargetingIntentMismatch { tick } => (tick, "targeting-intents"),
        RuntimeReplayError::TargetingQueryMismatch { tick } => (tick, "targeting-queries"),
        RuntimeReplayError::PhysicsQueryBatchMismatch { tick } => (tick, "physics-query-batch"),
        RuntimeReplayError::PhysicsQueryResultMismatch { tick } => (tick, "physics-query-results"),
        RuntimeReplayError::InteractionAvailabilityMismatch { tick } => {
            (tick, "interaction-availability")
        }
        _ => (fallback_tick, "replay-driver"),
    };
    ReplayError::RecordedStageMismatch { tick, stage }
}
