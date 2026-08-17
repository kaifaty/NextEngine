use std::time::Duration;

use next_application::ApplicationRunOutcomeV1;
use next_contracts::canonical::CanonicalDecodeLimits;
use next_contracts::command::{CommandPayload, WorldCommand};
use next_reference_game::ReferenceLiveStateV2;

use super::{
    ApplicationLongSessionReport, ApplicationMeasurement, DriverMeasurement, LONG_SESSION_TICKS,
    LiveRuntimePerformanceError, LiveRuntimePerformanceReport, LiveRuntimeWorkload,
    PreparedApplicationWorkload, STATE_SAMPLE_INTERVAL_TICKS, SYSTEMIC_RPG_COMMAND_BODY_COUNT,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct LiveCommandBodyCountsV1 {
    noop: u64,
    rpg: u64,
    physical: u64,
    world_routine: u64,
    world_population: u64,
    world_activity: u64,
    agent_cognition: u64,
}

impl LiveCommandBodyCountsV1 {
    fn total(self) -> Option<u64> {
        self.noop
            .checked_add(self.rpg)?
            .checked_add(self.physical)?
            .checked_add(self.world_routine)?
            .checked_add(self.world_population)?
            .checked_add(self.world_activity)?
            .checked_add(self.agent_cognition)
    }
}

fn live_command_body_counts(
    state: &ReferenceLiveStateV2,
) -> Result<LiveCommandBodyCountsV1, LiveRuntimePerformanceError> {
    let mut counts = LiveCommandBodyCountsV1::default();
    for bytes in state
        .checkpoint
        .runtime_snapshot
        .body_archive
        .entries()
        .values()
    {
        let command = WorldCommand::from_canonical_bytes(bytes, CanonicalDecodeLimits::default())
            .map_err(|error| {
            LiveRuntimePerformanceError::new("command body classification", error.to_string())
        })?;
        let count = match &command.body.payload {
            CommandPayload::Noop => &mut counts.noop,
            CommandPayload::Rpg(_) => &mut counts.rpg,
            CommandPayload::Physical(_) | CommandPayload::RootMotion(_) => &mut counts.physical,
            CommandPayload::WorldRoutine(_) => &mut counts.world_routine,
            CommandPayload::WorldPopulation(_) => &mut counts.world_population,
            CommandPayload::WorldActivity(_) => &mut counts.world_activity,
            CommandPayload::AgentCognition(_) => &mut counts.agent_cognition,
        };
        *count = count.checked_add(1).ok_or_else(|| {
            LiveRuntimePerformanceError::new("command body classification", "count overflow")
        })?;
    }
    Ok(counts)
}

pub(super) fn finalize_driver_measurement(
    measurement: DriverMeasurement,
    workload: LiveRuntimeWorkload,
) -> Result<LiveRuntimePerformanceReport, LiveRuntimePerformanceError> {
    let command_body_count = u64::try_from(
        measurement
            .state
            .checkpoint
            .runtime_snapshot
            .body_archive
            .entries()
            .len(),
    )
    .map_err(|error| LiveRuntimePerformanceError::new("command body count", error.to_string()))?;
    let command_body_counts = live_command_body_counts(&measurement.state)?;
    // One movement command is authored per measured tick. Each durable
    // World Services or cognition revision is backed by exactly one
    // additional command body in the ledger.
    let routine_command_body_count = measurement
        .state
        .world_routine_snapshot_or_none
        .as_ref()
        .map_or(0, |snapshot| snapshot.record.record_revision);
    let population_command_body_count = measurement
        .state
        .world_population_snapshot
        .records
        .iter()
        .try_fold(0_u64, |count, record| {
            count.checked_add(record.record_revision).ok_or_else(|| {
                LiveRuntimePerformanceError::new(
                    "command body count",
                    "population revision count overflow",
                )
            })
        })?;
    if measurement.state.agent_cognition_snapshot.revision
        != measurement.state.agent_memory_snapshot.revision
    {
        return Err(LiveRuntimePerformanceError::new(
            "cognition revision closure",
            "Agent and Memory revisions diverged",
        ));
    }
    let cognition_command_body_count = measurement.state.agent_cognition_snapshot.revision;
    let activity_command_body_count = measurement.state.world_activity_snapshot.record_revision;
    let expected_command_body_count = workload
        .ticks
        .checked_add(routine_command_body_count)
        .and_then(|count| count.checked_add(population_command_body_count))
        .and_then(|count| count.checked_add(activity_command_body_count))
        .and_then(|count| count.checked_add(cognition_command_body_count))
        .and_then(|count| count.checked_add(SYSTEMIC_RPG_COMMAND_BODY_COUNT))
        .ok_or_else(|| {
            LiveRuntimePerformanceError::new("command body count", "expected count overflow")
        })?;
    let final_state_root = next_contracts::snapshot::
        world_checkpoint_with_physical_animation_and_systemic_cognition_v1_state_root_from_canonical_components(
            &measurement.state.checkpoint_canonical_components,
            &measurement.state.world_streaming_snapshot,
            measurement.state.world_routine_snapshot_or_none.as_ref(),
            &measurement.state.world_population_snapshot,
            &measurement.state.world_activity_snapshot,
            &measurement.state.agent_cognition_snapshot,
            &measurement.state.agent_memory_snapshot,
            &measurement.state.physical_animation_snapshot,
        )
        .map_err(|error| {
            LiveRuntimePerformanceError::new("live application state root", error.to_string())
        })?;
    if measurement.state.ticks != workload.ticks
        || command_body_count != expected_command_body_count
        || command_body_counts.total() != Some(command_body_count)
        || command_body_counts.noop != 0
        || command_body_counts.physical != workload.ticks
        || command_body_counts.rpg != SYSTEMIC_RPG_COMMAND_BODY_COUNT
        || command_body_counts.world_routine != routine_command_body_count
        || command_body_counts.world_population != population_command_body_count
        || command_body_counts.world_activity != activity_command_body_count
        || command_body_counts.agent_cognition != cognition_command_body_count
        || measurement.checkpoint_root != measurement.state.checkpoint.state_root
    {
        return Err(LiveRuntimePerformanceError::new(
            "live runtime output",
            format!(
                "ticks={}, command_bodies={}, expected_command_bodies={}, classified={command_body_counts:?}, routine_revisions={}, population_revisions={}, activity_revisions={}, cognition_revisions={}, checkpoint_root={}, final_checkpoint_root={}, final_application_root={}",
                measurement.state.ticks,
                command_body_count,
                expected_command_body_count,
                routine_command_body_count,
                population_command_body_count,
                activity_command_body_count,
                cognition_command_body_count,
                measurement.checkpoint_root.to_hex(),
                measurement.state.checkpoint.state_root.to_hex(),
                final_state_root.to_hex()
            ),
        ));
    }
    let elapsed = Duration::from_micros(u64::try_from(measurement.elapsed_microseconds).map_err(
        |error| LiveRuntimePerformanceError::new("live runtime measurement", error.to_string()),
    )?);
    if elapsed > workload.time_limit {
        return Err(LiveRuntimePerformanceError::new(
            "live runtime performance threshold",
            format!(
                "elapsed={elapsed:?}, windows_us={:?}, limit={:?}",
                measurement.window_microseconds, workload.time_limit
            ),
        ));
    }
    Ok(LiveRuntimePerformanceReport {
        ticks: measurement.state.ticks,
        command_body_count,
        elapsed_microseconds: measurement.elapsed_microseconds,
        window_microseconds: measurement.window_microseconds,
        checkpoint_microseconds: measurement.checkpoint_microseconds,
        driver_prepare_microseconds: measurement.driver_prepare_microseconds,
        driver_commit_microseconds: measurement.driver_commit_microseconds,
        driver_checkpoint_materialization_microseconds: measurement
            .driver_checkpoint_materialization_microseconds,
        identity_index_root_probe_microseconds: measurement.identity_index_root_probe_microseconds,
        archive_root_probe_microseconds: measurement.archive_root_probe_microseconds,
        camera_event_count: measurement.camera_event_count,
        application_window_microseconds: [0; 3],
        application_sample_interval_microseconds: [0; 3],
        application_non_sample_tick_microseconds: Vec::new(),
        application_sample_tick_microseconds: Vec::new(),
        application_final_state_root: None,
        final_command_archive_root: measurement
            .state
            .checkpoint
            .runtime_snapshot
            .command_ledger
            .body_archive
            .archive_root,
        final_command_identity_index_root: measurement
            .state
            .checkpoint
            .runtime_snapshot
            .command_ledger
            .identity_index
            .index_root,
        final_state_root,
    })
}

pub(super) fn finalize_application_measurement(
    prepared: &PreparedApplicationWorkload,
    measurement: ApplicationMeasurement,
) -> Result<ApplicationLongSessionReport, LiveRuntimePerformanceError> {
    let run: ApplicationRunOutcomeV1 =
        prepared.application.current_live_run().map_err(|error| {
            LiveRuntimePerformanceError::new("final live application state", error.to_string())
        })?;
    if measurement.last_tick != LONG_SESSION_TICKS || run.ticks != LONG_SESSION_TICKS {
        return Err(LiveRuntimePerformanceError::new(
            "application long-session output",
            format!(
                "presentation_tick={}, authoritative_ticks={}",
                measurement.last_tick, run.ticks
            ),
        ));
    }
    let expected_ordinary =
        usize::try_from(LONG_SESSION_TICKS - LONG_SESSION_TICKS / STATE_SAMPLE_INTERVAL_TICKS)
            .map_err(|error| {
                LiveRuntimePerformanceError::new(
                    "application non-sample tick count",
                    error.to_string(),
                )
            })?;
    let expected_samples = usize::try_from(LONG_SESSION_TICKS / STATE_SAMPLE_INTERVAL_TICKS)
        .map_err(|error| {
            LiveRuntimePerformanceError::new("application interval sample count", error.to_string())
        })?;
    if measurement.non_sample_tick_microseconds.len() != expected_ordinary
        || measurement.sample_tick_microseconds.len() != expected_samples
    {
        return Err(LiveRuntimePerformanceError::new(
            "application per-tick measurement",
            format!(
                "non_sample_ticks={}, interval_samples={}",
                measurement.non_sample_tick_microseconds.len(),
                measurement.sample_tick_microseconds.len()
            ),
        ));
    }
    Ok(ApplicationLongSessionReport {
        ticks: run.ticks,
        window_microseconds: measurement.window_microseconds,
        sample_interval_microseconds: measurement.sample_interval_microseconds,
        non_sample_tick_microseconds: measurement.non_sample_tick_microseconds,
        sample_tick_microseconds: measurement.sample_tick_microseconds,
        authoritative_state_root: run.authoritative_state_root,
        command_archive_root: run.command_archive_root,
        command_identity_index_root: run.command_identity_index_root,
    })
}
