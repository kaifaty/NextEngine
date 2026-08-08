use std::mem;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use next_application::{
    ApplicationCoordinator, ApplicationRunOutcomeV1, FixedStepLiveSchedulerV1, LaunchRequestV1,
};
use next_assets::ContentStore;
use next_contracts::ids::{ContentHash, StateRoot};
use next_contracts::ledger::command_identity_index_root;
use next_contracts::platform::PlatformEventV1;
use next_contracts::session::{CompositionRootV1, PresentationTargetKindV1};
use next_reference_game::{ReferenceGameDriverV1, ReferenceLiveStateV1};

use crate::scratch::{ScratchContext, ScratchDirectory};

use super::{
    APPLICATION_ONE_TICK_ELAPSED, LONG_SESSION_CAMERA_INTERVAL_TICKS, LONG_SESSION_TICKS,
    LONG_SESSION_WINDOW_TICKS, LONG_SESSION_WORKLOAD, LiveRuntimePerformanceError,
    LiveRuntimePerformanceReport, LiveRuntimeWorkload, SMOKE_WORKLOAD, STATE_SAMPLE_INTERVAL_TICKS,
    camera_changed_event, camera_changed_event_for_host, movement_started_event,
    movement_started_event_for_host,
};

static NEXT_PREPARATION_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WorkloadKind {
    Smoke,
    LongSession,
}

/// A production-path live-runtime workload whose filesystem, project, driver,
/// application, inputs, and report buffers have already been prepared.
///
/// Call [`Self::run_measured`] only after opening an external measurement
/// window. Close that window before calling [`Self::finish`], which performs
/// final validation, report construction, object destruction, and scratch
/// cleanup.
pub struct PreparedLiveRuntimePerformanceCheck {
    preparation_id: u64,
    kind: WorkloadKind,
    workload: LiveRuntimeWorkload,
    driver: PreparedDriverWorkload,
    application: Option<PreparedApplicationWorkload>,
    driver_directory: ScratchDirectory,
    application_directory: Option<ScratchDirectory>,
    run_started: bool,
}

/// Opaque measured outcome. It is intentionally consumed by
/// [`PreparedLiveRuntimePerformanceCheck::finish`] after the external
/// measurement window has closed.
pub struct LiveRuntimePerformanceMeasurement {
    preparation_id: u64,
    kind: WorkloadKind,
    driver: DriverMeasurement,
    application: Option<ApplicationMeasurement>,
}

struct PreparedDriverWorkload {
    driver: ReferenceGameDriverV1,
    tick_events: Vec<Option<PlatformEventV1>>,
    window_microseconds: [u128; 3],
    checkpoint_microseconds: [u128; 3],
    driver_prepare_microseconds: Vec<u128>,
    driver_commit_microseconds: Vec<u128>,
    driver_checkpoint_materialization_microseconds: Vec<u128>,
    identity_index_root_probe_microseconds: [u128; 3],
    archive_root_probe_microseconds: [u128; 3],
    camera_event_count: u64,
}

struct DriverMeasurement {
    state: ReferenceLiveStateV1,
    checkpoint_root: StateRoot,
    elapsed_microseconds: u128,
    window_microseconds: [u128; 3],
    checkpoint_microseconds: [u128; 3],
    driver_prepare_microseconds: Vec<u128>,
    driver_commit_microseconds: Vec<u128>,
    driver_checkpoint_materialization_microseconds: Vec<u128>,
    identity_index_root_probe_microseconds: [u128; 3],
    archive_root_probe_microseconds: [u128; 3],
    camera_event_count: u64,
}

struct PreparedApplicationWorkload {
    application: ApplicationCoordinator,
    scheduler: FixedStepLiveSchedulerV1,
    callback_events: Vec<Option<PlatformEventV1>>,
    window_microseconds: [u128; 3],
    sample_interval_microseconds: [u128; 3],
    non_sample_tick_microseconds: Vec<u128>,
    sample_tick_microseconds: Vec<u128>,
}

struct ApplicationMeasurement {
    last_tick: u64,
    window_microseconds: [u128; 3],
    sample_interval_microseconds: [u128; 3],
    non_sample_tick_microseconds: Vec<u128>,
    sample_tick_microseconds: Vec<u128>,
}

struct ApplicationLongSessionReport {
    ticks: u64,
    window_microseconds: [u128; 3],
    sample_interval_microseconds: [u128; 3],
    non_sample_tick_microseconds: Vec<u128>,
    sample_tick_microseconds: Vec<u128>,
    authoritative_state_root: ContentHash,
    command_archive_root: ContentHash,
    command_identity_index_root: ContentHash,
}

pub fn prepare_live_runtime_performance_check()
-> Result<PreparedLiveRuntimePerformanceCheck, LiveRuntimePerformanceError> {
    prepare_live_runtime_performance_check_in(&std::env::temp_dir())
}

pub fn prepare_live_runtime_performance_check_in(
    scratch_root: &Path,
) -> Result<PreparedLiveRuntimePerformanceCheck, LiveRuntimePerformanceError> {
    PreparedLiveRuntimePerformanceCheck::prepare(scratch_root, SMOKE_WORKLOAD, WorkloadKind::Smoke)
}

pub fn prepare_live_runtime_long_session_performance_check()
-> Result<PreparedLiveRuntimePerformanceCheck, LiveRuntimePerformanceError> {
    prepare_live_runtime_long_session_performance_check_in(&std::env::temp_dir())
}

pub fn prepare_live_runtime_long_session_performance_check_in(
    scratch_root: &Path,
) -> Result<PreparedLiveRuntimePerformanceCheck, LiveRuntimePerformanceError> {
    PreparedLiveRuntimePerformanceCheck::prepare(
        scratch_root,
        LONG_SESSION_WORKLOAD,
        WorkloadKind::LongSession,
    )
}

impl PreparedLiveRuntimePerformanceCheck {
    fn prepare(
        scratch_root: &Path,
        workload: LiveRuntimeWorkload,
        kind: WorkloadKind,
    ) -> Result<Self, LiveRuntimePerformanceError> {
        let scratch = ScratchContext::new(scratch_root)
            .map_err(|error| LiveRuntimePerformanceError::new("scratch root", error.to_string()))?;
        let (driver, driver_directory) = PreparedDriverWorkload::prepare(&scratch, workload)?;
        let (application, application_directory) = if kind == WorkloadKind::LongSession {
            match PreparedApplicationWorkload::prepare(&scratch) {
                Ok((application, directory)) => (Some(application), Some(directory)),
                Err(error) => {
                    drop(driver);
                    return Err(finish_failed_preparation(
                        driver_directory,
                        error,
                        "remove performance fixture",
                    ));
                }
            }
        } else {
            (None, None)
        };
        let preparation_id = match next_preparation_id() {
            Ok(preparation_id) => preparation_id,
            Err(error) => {
                drop(application);
                drop(driver);
                let error = match application_directory {
                    Some(directory) => finish_failed_preparation(
                        directory,
                        error,
                        "remove application performance fixture",
                    ),
                    None => error,
                };
                return Err(finish_failed_preparation(
                    driver_directory,
                    error,
                    "remove performance fixture",
                ));
            }
        };
        Ok(Self {
            preparation_id,
            kind,
            workload,
            driver,
            application,
            driver_directory,
            application_directory,
            run_started: false,
        })
    }

    #[must_use]
    pub const fn ticks(&self) -> u64 {
        self.workload.ticks
    }

    /// Executes only the declared driver tick workload and, for the long-session
    /// scenario, the matching application callback workload.
    pub fn run_measured(
        &mut self,
    ) -> Result<LiveRuntimePerformanceMeasurement, LiveRuntimePerformanceError> {
        if self.run_started {
            return Err(LiveRuntimePerformanceError::new(
                "prepared live runtime workload",
                "measured workload can run only once",
            ));
        }
        self.run_started = true;
        let driver = self.driver.run_measured(self.workload)?;
        let application = self
            .application
            .as_mut()
            .map(PreparedApplicationWorkload::run_measured)
            .transpose()?;
        Ok(LiveRuntimePerformanceMeasurement {
            preparation_id: self.preparation_id,
            kind: self.kind,
            driver,
            application,
        })
    }

    /// Finalizes a measured result after the caller has closed its external
    /// measurement window. Scratch cleanup is attempted for both success and
    /// failure results.
    pub fn finish(
        self,
        measurement: Result<LiveRuntimePerformanceMeasurement, LiveRuntimePerformanceError>,
    ) -> Result<LiveRuntimePerformanceReport, LiveRuntimePerformanceError> {
        let result = measurement.and_then(|measurement| self.build_report(measurement));
        self.cleanup(result)
    }

    /// Cancels a prepared workload before measurement and reports cleanup
    /// failures instead of relying on best-effort `Drop` cleanup.
    pub fn cancel(self) -> Result<(), LiveRuntimePerformanceError> {
        self.cleanup(Ok(()))
    }

    fn build_report(
        &self,
        measurement: LiveRuntimePerformanceMeasurement,
    ) -> Result<LiveRuntimePerformanceReport, LiveRuntimePerformanceError> {
        if !self.run_started
            || measurement.preparation_id != self.preparation_id
            || measurement.kind != self.kind
        {
            return Err(LiveRuntimePerformanceError::new(
                "prepared live runtime workload",
                "measurement does not belong to this completed preparation",
            ));
        }
        let mut report = finalize_driver_measurement(measurement.driver, self.workload)?;
        match (
            self.application.as_ref(),
            measurement.application,
            self.kind,
        ) {
            (Some(application), Some(measurement), WorkloadKind::LongSession) => {
                let application = finalize_application_measurement(application, measurement)?;
                if application.ticks != report.ticks
                    || application.command_archive_root != report.final_command_archive_root
                    || application.command_identity_index_root
                        != report.final_command_identity_index_root
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
                report.application_sample_interval_microseconds =
                    application.sample_interval_microseconds;
                report.application_non_sample_tick_microseconds =
                    application.non_sample_tick_microseconds;
                report.application_sample_tick_microseconds = application.sample_tick_microseconds;
                report.application_final_state_root = Some(application.authoritative_state_root);
            }
            (None, None, WorkloadKind::Smoke) => {}
            _ => {
                return Err(LiveRuntimePerformanceError::new(
                    "prepared live runtime workload",
                    "application outcome does not match the prepared scenario",
                ));
            }
        }
        Ok(report)
    }

    fn cleanup<T>(
        self,
        result: Result<T, LiveRuntimePerformanceError>,
    ) -> Result<T, LiveRuntimePerformanceError> {
        let Self {
            driver,
            application,
            driver_directory,
            application_directory,
            ..
        } = self;
        drop(application);
        drop(driver);
        let result = match application_directory {
            Some(directory) => directory.finish(result, |error| {
                LiveRuntimePerformanceError::new(
                    "remove application performance fixture",
                    error.to_string(),
                )
            }),
            None => result,
        };
        driver_directory.finish(result, |error| {
            LiveRuntimePerformanceError::new("remove performance fixture", error.to_string())
        })
    }
}

impl PreparedDriverWorkload {
    fn prepare(
        scratch: &ScratchContext,
        workload: LiveRuntimeWorkload,
    ) -> Result<(Self, ScratchDirectory), LiveRuntimePerformanceError> {
        let source = next_reference_game::project_source_v2().map_err(|error| {
            LiveRuntimePerformanceError::new("fixture source", error.to_string())
        })?;
        let cooked = next_project::cook_project_v2(source)
            .map_err(|error| LiveRuntimePerformanceError::new("cook fixture", error.to_string()))?;
        let directory = scratch
            .create_directory(workload.directory_label)
            .map_err(|error| {
                LiveRuntimePerformanceError::new("create performance fixture", error.to_string())
            })?;
        let prepared = (|| {
            let store = ContentStore::new(directory.path());
            store
                .publish(&cooked.publication().map_err(|error| {
                    LiveRuntimePerformanceError::new("build publication", error.to_string())
                })?)
                .map_err(|error| {
                    LiveRuntimePerformanceError::new("publish fixture", error.to_string())
                })?;
            let project = next_project::activate_project_package(&store).map_err(|error| {
                LiveRuntimePerformanceError::new("activate fixture", error.to_string())
            })?;
            let driver = ReferenceGameDriverV1::new(project, true).map_err(|error| {
                LiveRuntimePerformanceError::new("create live driver", error.to_string())
            })?;
            let (tick_events, camera_event_count) = prepare_driver_inputs(workload)?;
            let tick_capacity = usize::try_from(workload.ticks).map_err(|error| {
                LiveRuntimePerformanceError::new("driver samples", error.to_string())
            })?;
            let checkpoint_capacity = usize::try_from(workload.ticks / STATE_SAMPLE_INTERVAL_TICKS)
                .map_err(|error| {
                    LiveRuntimePerformanceError::new("driver checkpoint samples", error.to_string())
                })?;
            Ok(Self {
                driver,
                tick_events,
                window_microseconds: [0; 3],
                checkpoint_microseconds: [0; 3],
                driver_prepare_microseconds: Vec::with_capacity(tick_capacity),
                driver_commit_microseconds: Vec::with_capacity(tick_capacity),
                driver_checkpoint_materialization_microseconds: Vec::with_capacity(
                    checkpoint_capacity,
                ),
                identity_index_root_probe_microseconds: [0; 3],
                archive_root_probe_microseconds: [0; 3],
                camera_event_count,
            })
        })();
        match prepared {
            Ok(prepared) => Ok((prepared, directory)),
            Err(error) => Err(finish_failed_preparation(
                directory,
                error,
                "remove performance fixture",
            )),
        }
    }

    fn run_measured(
        &mut self,
        workload: LiveRuntimeWorkload,
    ) -> Result<DriverMeasurement, LiveRuntimePerformanceError> {
        let mut window_started = Instant::now();
        let mut checkpoint_root = StateRoot::default();
        let mut final_state = None;
        for tick in 1..=workload.ticks {
            let event_index = usize::try_from(tick - 1).map_err(|error| {
                LiveRuntimePerformanceError::new("driver input index", error.to_string())
            })?;
            let events = self.tick_events[event_index].as_slice();
            let prepare_started = Instant::now();
            let prepared = self.driver.stage_advance(events).map_err(|error| {
                LiveRuntimePerformanceError::new("prepare live movement", error.to_string())
            })?;
            let validated = self
                .driver
                .validate_prepared_advance(prepared)
                .map_err(|error| {
                    LiveRuntimePerformanceError::new("validate live movement", error.to_string())
                })?;
            self.driver_prepare_microseconds
                .push(prepare_started.elapsed().as_micros());
            let commit_started = Instant::now();
            self.driver
                .commit_validated_advance(validated)
                .map_err(|error| {
                    LiveRuntimePerformanceError::new("commit live movement", error.to_string())
                })?;
            self.driver_commit_microseconds
                .push(commit_started.elapsed().as_micros());

            let mut checkpoint_state = None;
            if tick.is_multiple_of(STATE_SAMPLE_INTERVAL_TICKS) {
                let checkpoint_started = Instant::now();
                let state = self.driver.state().map_err(|error| {
                    LiveRuntimePerformanceError::new("build live checkpoint", error.to_string())
                })?;
                checkpoint_root = state.checkpoint.state_root;
                let window_index = window_index(tick, workload.window_ticks, "checkpoint")?;
                let checkpoint_elapsed = checkpoint_started.elapsed().as_micros();
                self.driver_checkpoint_materialization_microseconds
                    .push(checkpoint_elapsed);
                self.checkpoint_microseconds[window_index] = self.checkpoint_microseconds
                    [window_index]
                    .checked_add(checkpoint_elapsed)
                    .ok_or_else(|| {
                        LiveRuntimePerformanceError::new(
                            "checkpoint measurement",
                            "elapsed microseconds overflow",
                        )
                    })?;
                checkpoint_state = Some(state);
            }

            if tick.is_multiple_of(workload.window_ticks) {
                let window_index = window_index(tick, workload.window_ticks, "measurement")?;
                self.window_microseconds[window_index] = window_started.elapsed().as_micros();
                let state = checkpoint_state.as_ref().ok_or_else(|| {
                    LiveRuntimePerformanceError::new(
                        "measurement window",
                        "window boundary must also be a checkpoint boundary",
                    )
                })?;
                probe_checkpoint_roots(
                    state,
                    window_index,
                    &mut self.identity_index_root_probe_microseconds,
                    &mut self.archive_root_probe_microseconds,
                )?;
                window_started = Instant::now();
            }
            if tick == workload.ticks {
                final_state = checkpoint_state;
            }
        }
        let elapsed_microseconds =
            self.window_microseconds
                .iter()
                .try_fold(0_u128, |elapsed, window| {
                    elapsed.checked_add(*window).ok_or_else(|| {
                        LiveRuntimePerformanceError::new(
                            "live runtime measurement",
                            "elapsed microseconds overflow",
                        )
                    })
                })?;
        let state = final_state.ok_or_else(|| {
            LiveRuntimePerformanceError::new(
                "final live state",
                "workload did not finish on a checkpoint boundary",
            )
        })?;
        Ok(DriverMeasurement {
            state,
            checkpoint_root,
            elapsed_microseconds,
            window_microseconds: self.window_microseconds,
            checkpoint_microseconds: self.checkpoint_microseconds,
            driver_prepare_microseconds: mem::take(&mut self.driver_prepare_microseconds),
            driver_commit_microseconds: mem::take(&mut self.driver_commit_microseconds),
            driver_checkpoint_materialization_microseconds: mem::take(
                &mut self.driver_checkpoint_materialization_microseconds,
            ),
            identity_index_root_probe_microseconds: self.identity_index_root_probe_microseconds,
            archive_root_probe_microseconds: self.archive_root_probe_microseconds,
            camera_event_count: self.camera_event_count,
        })
    }
}

impl PreparedApplicationWorkload {
    fn prepare(
        scratch: &ScratchContext,
    ) -> Result<(Self, ScratchDirectory), LiveRuntimePerformanceError> {
        let directory = scratch
            .create_directory("live-application-long-session-performance")
            .map_err(|error| {
                LiveRuntimePerformanceError::new(
                    "create application performance fixture",
                    error.to_string(),
                )
            })?;
        let prepared = (|| {
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
            let staged = scheduler
                .advance_reference_game_presentation_shared(
                    &mut application,
                    Duration::ZERO,
                    std::slice::from_ref(&movement),
                )
                .map_err(|error| {
                    LiveRuntimePerformanceError::new(
                        "stage application movement input",
                        error.to_string(),
                    )
                })?;
            if staged.is_some() {
                return Err(LiveRuntimePerformanceError::new(
                    "stage application movement input",
                    "zero elapsed time advanced the live runtime",
                ));
            }
            let callback_events =
                prepare_application_inputs(host_instance_id, capabilities.canonical_hash)?;
            let ordinary_capacity = usize::try_from(
                LONG_SESSION_TICKS - LONG_SESSION_TICKS / STATE_SAMPLE_INTERVAL_TICKS,
            )
            .map_err(|error| {
                LiveRuntimePerformanceError::new("application non-sample ticks", error.to_string())
            })?;
            let sample_capacity = usize::try_from(LONG_SESSION_TICKS / STATE_SAMPLE_INTERVAL_TICKS)
                .map_err(|error| {
                    LiveRuntimePerformanceError::new(
                        "application interval samples",
                        error.to_string(),
                    )
                })?;
            Ok(Self {
                application,
                scheduler,
                callback_events,
                window_microseconds: [0; 3],
                sample_interval_microseconds: [0; 3],
                non_sample_tick_microseconds: Vec::with_capacity(ordinary_capacity),
                sample_tick_microseconds: Vec::with_capacity(sample_capacity),
            })
        })();
        match prepared {
            Ok(prepared) => Ok((prepared, directory)),
            Err(error) => Err(finish_failed_preparation(
                directory,
                error,
                "remove application performance fixture",
            )),
        }
    }

    fn run_measured(&mut self) -> Result<ApplicationMeasurement, LiveRuntimePerformanceError> {
        let mut last_tick = 0_u64;
        let mut window_started = Instant::now();
        for callback in 1..=LONG_SESSION_TICKS {
            let event_index = usize::try_from(callback - 1).map_err(|error| {
                LiveRuntimePerformanceError::new("application input index", error.to_string())
            })?;
            let events = self.callback_events[event_index].as_slice();
            let call_started = Instant::now();
            let presentation = self
                .scheduler
                .advance_reference_game_presentation_shared(
                    &mut self.application,
                    APPLICATION_ONE_TICK_ELAPSED,
                    events,
                )
                .map_err(|error| {
                    LiveRuntimePerformanceError::new("advance live application", error.to_string())
                })?;
            let call_microseconds = call_started.elapsed().as_micros();
            let Some(presentation) = presentation else {
                continue;
            };
            last_tick = presentation.simulation_tick;
            let window_index = window_index(
                last_tick,
                LONG_SESSION_WINDOW_TICKS,
                "application measurement",
            )?;
            if last_tick.is_multiple_of(STATE_SAMPLE_INTERVAL_TICKS) {
                self.sample_tick_microseconds.push(call_microseconds);
                self.sample_interval_microseconds[window_index] = self.sample_interval_microseconds
                    [window_index]
                    .checked_add(call_microseconds)
                    .ok_or_else(|| {
                        LiveRuntimePerformanceError::new(
                            "application interval-sample measurement",
                            "elapsed microseconds overflow",
                        )
                    })?;
            } else {
                self.non_sample_tick_microseconds.push(call_microseconds);
            }
            if last_tick.is_multiple_of(LONG_SESSION_WINDOW_TICKS) {
                self.window_microseconds[window_index] = window_started.elapsed().as_micros();
                window_started = Instant::now();
            }
        }
        Ok(ApplicationMeasurement {
            last_tick,
            window_microseconds: self.window_microseconds,
            sample_interval_microseconds: self.sample_interval_microseconds,
            non_sample_tick_microseconds: mem::take(&mut self.non_sample_tick_microseconds),
            sample_tick_microseconds: mem::take(&mut self.sample_tick_microseconds),
        })
    }
}

fn prepare_driver_inputs(
    workload: LiveRuntimeWorkload,
) -> Result<(Vec<Option<PlatformEventV1>>, u64), LiveRuntimePerformanceError> {
    let capacity = usize::try_from(workload.ticks).map_err(|error| {
        LiveRuntimePerformanceError::new("driver input count", error.to_string())
    })?;
    let movement = movement_started_event()?;
    let mut inputs = Vec::with_capacity(capacity);
    let mut source_sequence = 1_u64;
    let mut camera_event_count = 0_u64;
    for tick in 1..=workload.ticks {
        let event = if tick == 1 {
            Some(movement.clone())
        } else if workload
            .camera_interval_ticks
            .is_some_and(|interval| tick.is_multiple_of(interval))
        {
            let event = camera_changed_event(source_sequence, tick)?;
            source_sequence = source_sequence.checked_add(1).ok_or_else(|| {
                LiveRuntimePerformanceError::new(
                    "camera event sequence",
                    "source sequence overflow",
                )
            })?;
            camera_event_count = camera_event_count.checked_add(1).ok_or_else(|| {
                LiveRuntimePerformanceError::new("camera event count", "count overflow")
            })?;
            Some(event)
        } else {
            None
        };
        inputs.push(event);
    }
    Ok((inputs, camera_event_count))
}

fn prepare_application_inputs(
    host_instance_id: next_contracts::ids::PersistentId,
    capability_set_hash: ContentHash,
) -> Result<Vec<Option<PlatformEventV1>>, LiveRuntimePerformanceError> {
    let capacity = usize::try_from(LONG_SESSION_TICKS).map_err(|error| {
        LiveRuntimePerformanceError::new("application input count", error.to_string())
    })?;
    let mut inputs = Vec::with_capacity(capacity);
    let mut source_sequence = 1_u64;
    for callback in 1..=LONG_SESSION_TICKS {
        let event = if callback.is_multiple_of(LONG_SESSION_CAMERA_INTERVAL_TICKS) {
            let event = camera_changed_event_for_host(
                host_instance_id,
                capability_set_hash,
                source_sequence,
                callback,
            )?;
            source_sequence = source_sequence.checked_add(1).ok_or_else(|| {
                LiveRuntimePerformanceError::new(
                    "application camera sequence",
                    "source sequence overflow",
                )
            })?;
            Some(event)
        } else {
            None
        };
        inputs.push(event);
    }
    Ok(inputs)
}

fn probe_checkpoint_roots(
    state: &ReferenceLiveStateV1,
    window_index: usize,
    identity_microseconds: &mut [u128; 3],
    archive_microseconds: &mut [u128; 3],
) -> Result<(), LiveRuntimePerformanceError> {
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
    identity_microseconds[window_index] = identity_probe_started.elapsed().as_micros();
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
            LiveRuntimePerformanceError::new("command-body archive root probe", error.to_string())
        })?;
    archive_microseconds[window_index] = archive_probe_started.elapsed().as_micros();
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
    Ok(())
}

fn finalize_driver_measurement(
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
    if measurement.state.ticks != workload.ticks
        || command_body_count != workload.ticks
        || measurement.checkpoint_root != measurement.state.checkpoint.state_root
    {
        return Err(LiveRuntimePerformanceError::new(
            "live runtime output",
            format!(
                "ticks={}, command_bodies={}, checkpoint_root={}, final_root={}",
                measurement.state.ticks,
                command_body_count,
                measurement.checkpoint_root.to_hex(),
                measurement.state.checkpoint.state_root.to_hex()
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
        final_state_root: measurement.state.checkpoint.state_root,
    })
}

fn finalize_application_measurement(
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

fn window_index(
    tick: u64,
    window_ticks: u64,
    context: &'static str,
) -> Result<usize, LiveRuntimePerformanceError> {
    usize::try_from((tick - 1) / window_ticks)
        .map_err(|error| LiveRuntimePerformanceError::new(context, error.to_string()))
}

fn next_preparation_id() -> Result<u64, LiveRuntimePerformanceError> {
    NEXT_PREPARATION_ID
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            current.checked_add(1)
        })
        .map_err(|_| {
            LiveRuntimePerformanceError::new(
                "prepared live runtime workload",
                "preparation identity overflow",
            )
        })
}

fn finish_failed_preparation(
    directory: ScratchDirectory,
    error: LiveRuntimePerformanceError,
    cleanup_context: &'static str,
) -> LiveRuntimePerformanceError {
    let primary = error.to_string();
    match directory.finish(Err::<(), _>(error), |cleanup_error| {
        LiveRuntimePerformanceError::new(
            cleanup_context,
            format!("{primary}; cleanup: {cleanup_error}"),
        )
    }) {
        Err(error) => error,
        Ok(()) => unreachable!("an error result cannot become successful during cleanup"),
    }
}

#[cfg(test)]
mod tests;
