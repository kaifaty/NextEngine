use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::time::{Duration, Instant};

use next_application::{ApplicationCoordinator, FixedStepLiveSchedulerV1, LaunchRequestV1};
use next_assets::ContentStore;
use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, PersistentId, SchemaId, StateRoot};
use next_contracts::input::{
    KEYBOARD_DEVICE_CLASS_ID, KEYBOARD_W_CONTROL_PATH_ID, MOUSE_DELTA_CONTROL_PATH_ID,
    MOUSE_DEVICE_CLASS_ID,
};
use next_contracts::ledger::command_identity_index_root;
use next_contracts::platform::{
    NormalizedControlEventV1, NormalizedControlPhaseV1, PlatformEventKindV1,
    PlatformEventPayloadV1, PlatformEventV1,
};
use next_contracts::session::{CompositionRootV1, PresentationTargetKindV1};
use next_reference_game::ReferenceGameDriverV1;

use crate::scratch::ScratchContext;

const LIVE_MOVEMENT_TICKS: u64 = 900;
const MEASUREMENT_WINDOW_TICKS: u64 = 300;
const LONG_SESSION_TICKS: u64 = 3_600;
const LONG_SESSION_WINDOW_TICKS: u64 = 1_200;
const LONG_SESSION_CAMERA_INTERVAL_TICKS: u64 = 15;
const CHECKPOINT_INTERVAL_TICKS: u64 = 30;
#[cfg(not(debug_assertions))]
const LIVE_MOVEMENT_LIMIT: Duration = Duration::from_secs(30);
#[cfg(debug_assertions)]
const LIVE_MOVEMENT_LIMIT: Duration = Duration::from_secs(60);
#[cfg(not(debug_assertions))]
const LONG_SESSION_LIMIT: Duration = Duration::from_secs(90);
#[cfg(debug_assertions)]
const LONG_SESSION_LIMIT: Duration = Duration::from_secs(240);

#[derive(Clone, Copy)]
struct LiveRuntimeWorkload {
    directory_label: &'static str,
    ticks: u64,
    window_ticks: u64,
    camera_interval_ticks: Option<u64>,
    time_limit: Duration,
}

const SMOKE_WORKLOAD: LiveRuntimeWorkload = LiveRuntimeWorkload {
    directory_label: "live-runtime-performance",
    ticks: LIVE_MOVEMENT_TICKS,
    window_ticks: MEASUREMENT_WINDOW_TICKS,
    camera_interval_ticks: None,
    time_limit: LIVE_MOVEMENT_LIMIT,
};

const LONG_SESSION_WORKLOAD: LiveRuntimeWorkload = LiveRuntimeWorkload {
    directory_label: "live-runtime-long-session-performance",
    ticks: LONG_SESSION_TICKS,
    window_ticks: LONG_SESSION_WINDOW_TICKS,
    camera_interval_ticks: Some(LONG_SESSION_CAMERA_INTERVAL_TICKS),
    time_limit: LONG_SESSION_LIMIT,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveRuntimePerformanceReport {
    pub ticks: u64,
    pub command_body_count: u64,
    pub elapsed_microseconds: u128,
    pub window_microseconds: [u128; 3],
    pub checkpoint_microseconds: [u128; 3],
    pub identity_index_root_probe_microseconds: [u128; 3],
    pub archive_root_probe_microseconds: [u128; 3],
    pub camera_event_count: u64,
    pub application_window_microseconds: [u128; 3],
    pub application_checkpoint_microseconds: [u128; 3],
    pub application_final_state_root: Option<ContentHash>,
    pub final_command_archive_root: ContentHash,
    pub final_command_identity_index_root: ContentHash,
    pub final_state_root: StateRoot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveRuntimePerformanceError {
    context: &'static str,
    detail: String,
}

impl LiveRuntimePerformanceError {
    fn new(context: &'static str, detail: impl Into<String>) -> Self {
        Self {
            context,
            detail: detail.into(),
        }
    }
}

impl Display for LiveRuntimePerformanceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.context, self.detail)
    }
}

impl Error for LiveRuntimePerformanceError {}

pub fn run_live_runtime_performance_check()
-> Result<LiveRuntimePerformanceReport, LiveRuntimePerformanceError> {
    run_live_runtime_performance_check_in(&std::env::temp_dir())
}

pub fn run_live_runtime_performance_check_in(
    scratch_root: &Path,
) -> Result<LiveRuntimePerformanceReport, LiveRuntimePerformanceError> {
    run_live_runtime_workload_in(scratch_root, SMOKE_WORKLOAD)
}

pub fn run_live_runtime_long_session_performance_check()
-> Result<LiveRuntimePerformanceReport, LiveRuntimePerformanceError> {
    run_live_runtime_long_session_performance_check_in(&std::env::temp_dir())
}

pub fn run_live_runtime_long_session_performance_check_in(
    scratch_root: &Path,
) -> Result<LiveRuntimePerformanceReport, LiveRuntimePerformanceError> {
    let mut report = run_live_runtime_workload_in(scratch_root, LONG_SESSION_WORKLOAD)?;
    let application = run_live_application_long_session_in(scratch_root)?;
    if application.ticks != report.ticks
        || application.command_archive_root != report.final_command_archive_root
        || application.command_identity_index_root != report.final_command_identity_index_root
    {
        return Err(LiveRuntimePerformanceError::new(
            "application long-session parity",
            format!(
                "driver_ticks={}, application_ticks={}, driver_archive={}, application_archive={}, driver_identity={}, application_identity={}",
                report.ticks,
                application.ticks,
                report.final_command_archive_root.to_hex(),
                application.command_archive_root.to_hex(),
                report.final_command_identity_index_root.to_hex(),
                application.command_identity_index_root.to_hex(),
            ),
        ));
    }
    report.application_window_microseconds = application.window_microseconds;
    report.application_checkpoint_microseconds = application.checkpoint_microseconds;
    report.application_final_state_root = Some(application.authoritative_state_root);
    Ok(report)
}

fn run_live_runtime_workload_in(
    scratch_root: &Path,
    workload: LiveRuntimeWorkload,
) -> Result<LiveRuntimePerformanceReport, LiveRuntimePerformanceError> {
    let scratch = ScratchContext::new(scratch_root)
        .map_err(|error| LiveRuntimePerformanceError::new("scratch root", error.to_string()))?;
    run_live_runtime_performance_check_with_scratch(&scratch, workload)
}

fn run_live_runtime_performance_check_with_scratch(
    scratch: &ScratchContext,
    workload: LiveRuntimeWorkload,
) -> Result<LiveRuntimePerformanceReport, LiveRuntimePerformanceError> {
    let source = next_reference_game::project_source_v2()
        .map_err(|error| LiveRuntimePerformanceError::new("fixture source", error.to_string()))?;
    let cooked = next_project::cook_project_v1(source)
        .map_err(|error| LiveRuntimePerformanceError::new("cook fixture", error.to_string()))?;
    let directory = scratch
        .create_directory(workload.directory_label)
        .map_err(|error| {
            LiveRuntimePerformanceError::new("create performance fixture", error.to_string())
        })?;
    let store = ContentStore::new(directory.path());
    let result = (|| {
        store
            .publish(&cooked.publication().map_err(|error| {
                LiveRuntimePerformanceError::new("build publication", error.to_string())
            })?)
            .map_err(|error| {
                LiveRuntimePerformanceError::new("publish fixture", error.to_string())
            })?;
        let project = next_project::activate_project(&store).map_err(|error| {
            LiveRuntimePerformanceError::new("activate fixture", error.to_string())
        })?;
        let mut driver = ReferenceGameDriverV1::new(project, true).map_err(|error| {
            LiveRuntimePerformanceError::new("create live driver", error.to_string())
        })?;
        let movement = movement_started_event()?;

        let started = Instant::now();
        let mut window_started = started;
        let mut window_microseconds = [0_u128; 3];
        let mut checkpoint_microseconds = [0_u128; 3];
        let mut identity_index_root_probe_microseconds = [0_u128; 3];
        let mut archive_root_probe_microseconds = [0_u128; 3];
        let mut checkpoint_root = StateRoot::default();
        let mut camera_event_count = 0_u64;
        let mut source_sequence = 1_u64;
        for tick in 1..=workload.ticks {
            let mut events = Vec::with_capacity(2);
            if tick == 1 {
                events.push(movement.clone());
            }
            if workload
                .camera_interval_ticks
                .is_some_and(|interval| tick.is_multiple_of(interval))
            {
                events.push(camera_changed_event(source_sequence, tick)?);
                source_sequence = source_sequence.checked_add(1).ok_or_else(|| {
                    LiveRuntimePerformanceError::new(
                        "camera event sequence",
                        "source sequence overflow",
                    )
                })?;
                camera_event_count = camera_event_count.checked_add(1).ok_or_else(|| {
                    LiveRuntimePerformanceError::new("camera event count", "count overflow")
                })?;
            }
            driver.advance(&events).map_err(|error| {
                LiveRuntimePerformanceError::new("advance live movement", error.to_string())
            })?;
            let mut checkpoint_state = None;
            if tick.is_multiple_of(CHECKPOINT_INTERVAL_TICKS) {
                let checkpoint_started = Instant::now();
                let state = driver.state().map_err(|error| {
                    LiveRuntimePerformanceError::new("build live checkpoint", error.to_string())
                })?;
                checkpoint_root = state.checkpoint.state_root;
                let window_index =
                    usize::try_from((tick - 1) / workload.window_ticks).map_err(|error| {
                        LiveRuntimePerformanceError::new(
                            "checkpoint measurement window",
                            error.to_string(),
                        )
                    })?;
                checkpoint_microseconds[window_index] = checkpoint_microseconds[window_index]
                    .checked_add(checkpoint_started.elapsed().as_micros())
                    .ok_or_else(|| {
                        LiveRuntimePerformanceError::new(
                            "checkpoint measurement",
                            "elapsed microseconds overflow",
                        )
                    })?;
                checkpoint_state = Some(state);
            }
            if tick.is_multiple_of(workload.window_ticks) {
                let window_index = usize::try_from(
                    tick.checked_div(workload.window_ticks)
                        .and_then(|value| value.checked_sub(1))
                        .ok_or_else(|| {
                            LiveRuntimePerformanceError::new(
                                "measurement window",
                                "window index underflow",
                            )
                        })?,
                )
                .map_err(|error| {
                    LiveRuntimePerformanceError::new("measurement window", error.to_string())
                })?;
                window_microseconds[window_index] = window_started.elapsed().as_micros();

                let state = checkpoint_state.as_ref().ok_or_else(|| {
                    LiveRuntimePerformanceError::new(
                        "measurement window",
                        "window boundary must also be a checkpoint boundary",
                    )
                })?;
                let identity_probe_started = Instant::now();
                let probed_identity_root = command_identity_index_root(
                    &state
                        .checkpoint
                        .runtime_snapshot
                        .command_ledger
                        .identity_index
                        .body,
                )
                .map_err(|error| {
                    LiveRuntimePerformanceError::new("identity-index root probe", error.to_string())
                })?;
                identity_index_root_probe_microseconds[window_index] =
                    identity_probe_started.elapsed().as_micros();
                if probed_identity_root
                    != state
                        .checkpoint
                        .runtime_snapshot
                        .command_ledger
                        .identity_index
                        .index_root
                {
                    return Err(LiveRuntimePerformanceError::new(
                        "identity-index root probe",
                        "recomputed root differs from the published root",
                    ));
                }

                let archive_probe_started = Instant::now();
                let probed_archive = state
                    .checkpoint
                    .runtime_snapshot
                    .body_archive
                    .manifest()
                    .map_err(|error| {
                        LiveRuntimePerformanceError::new(
                            "command-body archive root probe",
                            error.to_string(),
                        )
                    })?;
                archive_root_probe_microseconds[window_index] =
                    archive_probe_started.elapsed().as_micros();
                if probed_archive
                    != state
                        .checkpoint
                        .runtime_snapshot
                        .command_ledger
                        .body_archive
                {
                    return Err(LiveRuntimePerformanceError::new(
                        "command-body archive root probe",
                        "recomputed manifest differs from the published manifest",
                    ));
                }
                window_started = Instant::now();
            }
        }
        let measured_elapsed_microseconds =
            window_microseconds
                .iter()
                .try_fold(0_u128, |elapsed, window| {
                    elapsed.checked_add(*window).ok_or_else(|| {
                        LiveRuntimePerformanceError::new(
                            "live runtime measurement",
                            "elapsed microseconds overflow",
                        )
                    })
                })?;
        let elapsed = Duration::from_micros(u64::try_from(measured_elapsed_microseconds).map_err(
            |error| LiveRuntimePerformanceError::new("live runtime measurement", error.to_string()),
        )?);
        let state = driver.state().map_err(|error| {
            LiveRuntimePerformanceError::new("final live state", error.to_string())
        })?;
        let command_body_count = u64::try_from(
            state
                .checkpoint
                .runtime_snapshot
                .body_archive
                .entries()
                .len(),
        )
        .map_err(|error| {
            LiveRuntimePerformanceError::new("command body count", error.to_string())
        })?;
        if state.ticks != workload.ticks
            || command_body_count != workload.ticks
            || checkpoint_root != state.checkpoint.state_root
        {
            return Err(LiveRuntimePerformanceError::new(
                "live runtime output",
                format!(
                    "ticks={}, command_bodies={}, checkpoint_root={}, final_root={}",
                    state.ticks,
                    command_body_count,
                    checkpoint_root.to_hex(),
                    state.checkpoint.state_root.to_hex()
                ),
            ));
        }
        if elapsed > workload.time_limit {
            return Err(LiveRuntimePerformanceError::new(
                "live runtime performance threshold",
                format!(
                    "elapsed={elapsed:?}, windows_us={window_microseconds:?}, limit={:?}",
                    workload.time_limit
                ),
            ));
        }
        Ok(LiveRuntimePerformanceReport {
            ticks: state.ticks,
            command_body_count,
            elapsed_microseconds: measured_elapsed_microseconds,
            window_microseconds,
            checkpoint_microseconds,
            identity_index_root_probe_microseconds,
            archive_root_probe_microseconds,
            camera_event_count,
            application_window_microseconds: [0; 3],
            application_checkpoint_microseconds: [0; 3],
            application_final_state_root: None,
            final_command_archive_root: state
                .checkpoint
                .runtime_snapshot
                .command_ledger
                .body_archive
                .archive_root,
            final_command_identity_index_root: state
                .checkpoint
                .runtime_snapshot
                .command_ledger
                .identity_index
                .index_root,
            final_state_root: state.checkpoint.state_root,
        })
    })();
    directory.finish(result, |error| {
        LiveRuntimePerformanceError::new("remove performance fixture", error.to_string())
    })
}

struct ApplicationLongSessionReport {
    ticks: u64,
    window_microseconds: [u128; 3],
    checkpoint_microseconds: [u128; 3],
    authoritative_state_root: ContentHash,
    command_archive_root: ContentHash,
    command_identity_index_root: ContentHash,
}

fn run_live_application_long_session_in(
    scratch_root: &Path,
) -> Result<ApplicationLongSessionReport, LiveRuntimePerformanceError> {
    let scratch = ScratchContext::new(scratch_root)
        .map_err(|error| LiveRuntimePerformanceError::new("scratch root", error.to_string()))?;
    let directory = scratch
        .create_directory("live-application-long-session-performance")
        .map_err(|error| {
            LiveRuntimePerformanceError::new(
                "create application performance fixture",
                error.to_string(),
            )
        })?;
    let result = (|| {
        let launch = LaunchRequestV1::reference(
            directory.path(),
            CompositionRootV1::Game,
            PresentationTargetKindV1::Interactive,
        );
        let capabilities = launch.platform_capability_set.clone().ok_or_else(|| {
            LiveRuntimePerformanceError::new(
                "application platform capabilities",
                "interactive launch did not declare capabilities",
            )
        })?;
        let mut application = ApplicationCoordinator::launch(launch).map_err(|error| {
            LiveRuntimePerformanceError::new("launch live application", error.to_string())
        })?;
        let host_instance_id =
            application
                .register_platform_host(&capabilities)
                .map_err(|error| {
                    LiveRuntimePerformanceError::new(
                        "register application platform host",
                        error.to_string(),
                    )
                })?;
        application
            .begin_reference_game_live(true)
            .map_err(|error| {
                LiveRuntimePerformanceError::new("begin live application", error.to_string())
            })?;
        let movement =
            movement_started_event_for_host(host_instance_id, capabilities.canonical_hash)?;
        let mut scheduler = FixedStepLiveSchedulerV1::reference_game_v1();
        let mut source_sequence = 1_u64;
        let mut previous_elapsed_nanos = 0_u64;
        let mut last_tick = 0_u64;
        let mut window_started = Instant::now();
        let mut window_microseconds = [0_u128; 3];
        let mut checkpoint_microseconds = [0_u128; 3];
        for callback in 1..=LONG_SESSION_TICKS {
            let mut events = Vec::with_capacity(2);
            if callback == 1 {
                events.push(movement.clone());
            }
            if callback.is_multiple_of(LONG_SESSION_CAMERA_INTERVAL_TICKS) {
                events.push(camera_changed_event_for_host(
                    host_instance_id,
                    capabilities.canonical_hash,
                    source_sequence,
                    callback,
                )?);
                source_sequence = source_sequence.checked_add(1).ok_or_else(|| {
                    LiveRuntimePerformanceError::new(
                        "application camera sequence",
                        "source sequence overflow",
                    )
                })?;
            }
            let total_elapsed_nanos = callback
                .checked_mul(1_000_000_000)
                .and_then(|value| value.checked_div(30))
                .ok_or_else(|| {
                    LiveRuntimePerformanceError::new(
                        "application scheduler interval",
                        "elapsed nanoseconds overflow",
                    )
                })?;
            let elapsed = Duration::from_nanos(
                total_elapsed_nanos
                    .checked_sub(previous_elapsed_nanos)
                    .ok_or_else(|| {
                        LiveRuntimePerformanceError::new(
                            "application scheduler interval",
                            "elapsed nanoseconds underflow",
                        )
                    })?,
            );
            previous_elapsed_nanos = total_elapsed_nanos;
            let call_started = Instant::now();
            let presentation = scheduler
                .advance_reference_game_presentation(&mut application, elapsed, &events)
                .map_err(|error| {
                    LiveRuntimePerformanceError::new("advance live application", error.to_string())
                })?;
            let call_microseconds = call_started.elapsed().as_micros();
            let Some(presentation) = presentation else {
                continue;
            };
            last_tick = presentation.simulation_tick;
            let window_index = usize::try_from((last_tick - 1) / LONG_SESSION_WINDOW_TICKS)
                .map_err(|error| {
                    LiveRuntimePerformanceError::new(
                        "application measurement window",
                        error.to_string(),
                    )
                })?;
            if last_tick.is_multiple_of(CHECKPOINT_INTERVAL_TICKS) {
                checkpoint_microseconds[window_index] = checkpoint_microseconds[window_index]
                    .checked_add(call_microseconds)
                    .ok_or_else(|| {
                        LiveRuntimePerformanceError::new(
                            "application checkpoint measurement",
                            "elapsed microseconds overflow",
                        )
                    })?;
            }
            if last_tick.is_multiple_of(LONG_SESSION_WINDOW_TICKS) {
                window_microseconds[window_index] = window_started.elapsed().as_micros();
                window_started = Instant::now();
            }
        }
        let run = application.current_live_run().map_err(|error| {
            LiveRuntimePerformanceError::new("final live application state", error.to_string())
        })?;
        if last_tick != LONG_SESSION_TICKS || run.ticks != LONG_SESSION_TICKS {
            return Err(LiveRuntimePerformanceError::new(
                "application long-session output",
                format!(
                    "presentation_tick={last_tick}, authoritative_ticks={}",
                    run.ticks
                ),
            ));
        }
        drop(application);
        Ok(ApplicationLongSessionReport {
            ticks: run.ticks,
            window_microseconds,
            checkpoint_microseconds,
            authoritative_state_root: run.authoritative_state_root,
            command_archive_root: run.command_archive_root,
            command_identity_index_root: run.command_identity_index_root,
        })
    })();
    directory.finish(result, |error| {
        LiveRuntimePerformanceError::new(
            "remove application performance fixture",
            error.to_string(),
        )
    })
}

fn camera_changed_event(
    source_sequence: u64,
    sample_tick: u64,
) -> Result<PlatformEventV1, LiveRuntimePerformanceError> {
    let yaw = if source_sequence.is_multiple_of(2) {
        -12
    } else {
        12
    };
    let control = NormalizedControlEventV1::new(
        SchemaId::new(MOUSE_DEVICE_CLASS_ID)
            .map_err(|error| LiveRuntimePerformanceError::new("device class", error.to_string()))?,
        PersistentId::from_bytes([0x74; 16]),
        SchemaId::new(MOUSE_DELTA_CONTROL_PATH_ID)
            .map_err(|error| LiveRuntimePerformanceError::new("control path", error.to_string()))?,
        NormalizedControlPhaseV1::Changed,
        vec![yaw, -7],
        Vec::new(),
        sample_tick,
        source_sequence,
    )
    .map_err(|error| LiveRuntimePerformanceError::new("camera control", error.to_string()))?;
    PlatformEventV1::new(
        PersistentId::from_bytes([0x75; 16]),
        SchemaId::new("nextengine.platform.source.live-runtime-performance")
            .map_err(|error| LiveRuntimePerformanceError::new("source id", error.to_string()))?,
        source_sequence,
        sample_tick,
        PlatformEventKindV1::Control,
        PlatformEventPayloadV1::Control(control),
        ContentHash::from_bytes(sha256(
            b"nextengine.platform.live-runtime-performance-capabilities.v1",
        )),
    )
    .map_err(|error| LiveRuntimePerformanceError::new("camera event", error.to_string()))
}

fn movement_started_event_for_host(
    host_instance_id: PersistentId,
    capability_set_hash: ContentHash,
) -> Result<PlatformEventV1, LiveRuntimePerformanceError> {
    control_event_for_host(
        host_instance_id,
        capability_set_hash,
        KEYBOARD_DEVICE_CLASS_ID,
        KEYBOARD_W_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Started,
        vec![i16::MAX],
        0,
        0,
    )
}

fn camera_changed_event_for_host(
    host_instance_id: PersistentId,
    capability_set_hash: ContentHash,
    source_sequence: u64,
    sample_tick: u64,
) -> Result<PlatformEventV1, LiveRuntimePerformanceError> {
    let yaw = if source_sequence.is_multiple_of(2) {
        -12
    } else {
        12
    };
    control_event_for_host(
        host_instance_id,
        capability_set_hash,
        MOUSE_DEVICE_CLASS_ID,
        MOUSE_DELTA_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Changed,
        vec![yaw, -7],
        sample_tick,
        source_sequence,
    )
}

#[allow(
    clippy::too_many_arguments,
    reason = "the helper mirrors the complete normalized platform control identity"
)]
fn control_event_for_host(
    host_instance_id: PersistentId,
    capability_set_hash: ContentHash,
    device_class: &str,
    control_path: &str,
    phase: NormalizedControlPhaseV1,
    value: Vec<i16>,
    sample_tick: u64,
    source_sequence: u64,
) -> Result<PlatformEventV1, LiveRuntimePerformanceError> {
    let control = NormalizedControlEventV1::new(
        SchemaId::new(device_class)
            .map_err(|error| LiveRuntimePerformanceError::new("device class", error.to_string()))?,
        PersistentId::from_bytes([0x74; 16]),
        SchemaId::new(control_path)
            .map_err(|error| LiveRuntimePerformanceError::new("control path", error.to_string()))?,
        phase,
        value,
        Vec::new(),
        sample_tick,
        source_sequence,
    )
    .map_err(|error| LiveRuntimePerformanceError::new("host control", error.to_string()))?;
    PlatformEventV1::new(
        host_instance_id,
        SchemaId::new("nextengine.platform.source.live-runtime-performance")
            .map_err(|error| LiveRuntimePerformanceError::new("source id", error.to_string()))?,
        source_sequence,
        sample_tick,
        PlatformEventKindV1::Control,
        PlatformEventPayloadV1::Control(control),
        capability_set_hash,
    )
    .map_err(|error| LiveRuntimePerformanceError::new("host control event", error.to_string()))
}

fn movement_started_event() -> Result<PlatformEventV1, LiveRuntimePerformanceError> {
    let control = NormalizedControlEventV1::new(
        SchemaId::new(KEYBOARD_DEVICE_CLASS_ID)
            .map_err(|error| LiveRuntimePerformanceError::new("device class", error.to_string()))?,
        PersistentId::from_bytes([0x74; 16]),
        SchemaId::new(KEYBOARD_W_CONTROL_PATH_ID)
            .map_err(|error| LiveRuntimePerformanceError::new("control path", error.to_string()))?,
        NormalizedControlPhaseV1::Started,
        vec![i16::MAX],
        Vec::new(),
        0,
        0,
    )
    .map_err(|error| LiveRuntimePerformanceError::new("movement control", error.to_string()))?;
    PlatformEventV1::new(
        PersistentId::from_bytes([0x75; 16]),
        SchemaId::new("nextengine.platform.source.live-runtime-performance")
            .map_err(|error| LiveRuntimePerformanceError::new("source id", error.to_string()))?,
        0,
        0,
        PlatformEventKindV1::Control,
        PlatformEventPayloadV1::Control(control),
        ContentHash::from_bytes(sha256(
            b"nextengine.platform.live-runtime-performance-capabilities.v1",
        )),
    )
    .map_err(|error| LiveRuntimePerformanceError::new("movement event", error.to_string()))
}

#[cfg(test)]
mod tests {
    #[test]
    fn sustained_live_movement_remains_within_the_declared_runtime_budget() {
        let report =
            super::run_live_runtime_performance_check().expect("live runtime performance gate");
        println!("{report:?}");
        assert_eq!(report.ticks, 900);
        assert_eq!(report.command_body_count, 900);
        assert_ne!(
            report.final_state_root,
            next_contracts::ids::StateRoot::default()
        );
    }
}
