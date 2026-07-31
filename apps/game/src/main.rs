#![forbid(unsafe_code)]

#[cfg(feature = "desktop-sdl-ash")]
use std::sync::mpsc;
#[cfg(any(feature = "desktop-sdl-ash", test))]
use std::sync::{
    Arc, RwLock,
    mpsc::{Receiver, SyncSender},
};
#[cfg(any(feature = "desktop-sdl-ash", test))]
use std::time::Duration;

use next_application::{
    ApplicationCloseOutcomeV1, ApplicationCoordinator, DiagnosticContextV1, DiagnosticReportV1,
    LaunchRequestV1, ProjectSelectionV1, RunReportV1, default_user_state_root,
};
#[cfg(any(feature = "desktop-sdl-ash", test))]
use next_application::{ApplicationError, FixedStepLiveSchedulerV1};
use next_contracts::session::{CompositionRootV1, PresentationTargetKindV1};

mod cli;

use cli::{AppFailure, GameOptions};

fn main() {
    match run(std::env::args().skip(1)) {
        Ok(report) => {
            println!("{}", report.to_json().expect("run report serializes"));
        }
        Err(error) => {
            eprintln!("next_game: {}", error.message);
            let report =
                DiagnosticReportV1::new(error.code, DiagnosticContextV1::message(&error.message));
            println!("{}", report.to_json().expect("diagnostic serializes"));
            std::process::exit(error.exit_code);
        }
    }
}

fn run(arguments: impl Iterator<Item = String>) -> Result<RunReportV1, AppFailure> {
    let options = GameOptions::parse(arguments)?;
    if options.help {
        eprintln!(
            "usage: next_game [--interactive [--maximum-frames <positive-integer>]] [--project <cooked-store>] [--lock <sha256>] [--state-root <directory>]"
        );
        return Err(AppFailure::help());
    }
    if options.interactive && !cfg!(feature = "desktop-sdl-ash") {
        return Err(AppFailure::cli(
            "PLATFORM_INTERACTIVE_ADAPTER_UNAVAILABLE",
            "interactive desktop adapter is not enabled",
        ));
    }
    let state_root = options
        .state_root
        .unwrap_or(default_user_state_root("game").map_err(AppFailure::application)?);
    let target = if options.interactive {
        PresentationTargetKindV1::Interactive
    } else {
        PresentationTargetKindV1::None
    };
    #[cfg(feature = "desktop-sdl-ash")]
    let platform_capability_set = if options.interactive {
        Some(
            next_desktop_sdl_ash::desktop_capability_set()
                .map_err(|error| AppFailure::cli(error.diagnostic_code(), error.to_string()))?,
        )
    } else {
        None
    };
    #[cfg(not(feature = "desktop-sdl-ash"))]
    let platform_capability_set = None;
    let launch = LaunchRequestV1 {
        project: options.project.map_or(
            ProjectSelectionV1::Reference,
            ProjectSelectionV1::PublishedStateRoot,
        ),
        expected_project_lock: options.expected_lock,
        state_root,
        composition_root: CompositionRootV1::Game,
        presentation_target: target,
        platform_capability_set,
    };
    if options.interactive {
        return run_interactive_session(launch, options.maximum_frames);
    }

    let mut application =
        ApplicationCoordinator::launch_or_resume(launch).map_err(AppFailure::application)?;
    eprintln!(
        "next_game: session {} active",
        application.state().session_id.to_hex()
    );
    let run = application
        .run_reference_game(true)
        .map_err(AppFailure::application)?;
    let close_options = next_application::CloseExecutionOptionsV1::default();
    let close = application
        .close(close_options)
        .map_err(AppFailure::application)?;
    if !matches!(close, ApplicationCloseOutcomeV1::Closed { .. }) {
        return Err(AppFailure::cli(
            "SESSION_FINAL_SAVE_FAILED",
            "application close did not reach a terminal receipt",
        ));
    }
    RunReportV1::new(CompositionRootV1::Game, &run, &close, 0).ok_or_else(|| {
        AppFailure::cli(
            "SESSION_TERMINAL_RECEIPT_MISSING",
            "closed application has no terminal receipt",
        )
    })
}

#[cfg(any(feature = "desktop-sdl-ash", test))]
fn begin_or_resume_reference_game_live(
    application: &mut ApplicationCoordinator,
) -> Result<next_application::ApplicationRunOutcomeV1, AppFailure> {
    match application.current_live_run() {
        Ok(run) => Ok(run),
        Err(ApplicationError::NoLiveRun) => application
            .begin_reference_game_live(true)
            .map_err(AppFailure::application),
        Err(error) => Err(AppFailure::application(error)),
    }
}

#[cfg(feature = "desktop-sdl-ash")]
fn run_interactive_session(
    launch: LaunchRequestV1,
    maximum_frames: Option<u64>,
) -> Result<RunReportV1, AppFailure> {
    let capabilities = launch.platform_capability_set.clone().ok_or_else(|| {
        AppFailure::cli(
            "PLATFORM_CAPABILITY_SET_REQUIRED",
            "interactive launch requires a platform capability set",
        )
    })?;
    let latest_snapshot = Arc::new(RwLock::new(None));
    let (work_sender, work_receiver) = mpsc::sync_channel(INTERACTIVE_SIMULATION_QUEUE_CAPACITY);
    let (failure_sender, failure_receiver) = mpsc::sync_channel(1);
    let (ready_sender, ready_receiver) = mpsc::sync_channel(1);
    let worker_snapshot = Arc::clone(&latest_snapshot);
    let worker = std::thread::Builder::new()
        .name("next-simulation".to_owned())
        .spawn(move || {
            run_interactive_simulation_session_worker(
                launch,
                capabilities,
                work_receiver,
                worker_snapshot,
                ready_sender,
                failure_sender,
            )
        })
        .map_err(|error| {
            AppFailure::cli(
                "SESSION_RUNTIME_FAILED",
                format!("failed to start the simulation worker: {error}"),
            )
        })?;

    let ready = match ready_receiver.recv() {
        Ok(Ok(ready)) => ready,
        Ok(Err(failure)) => {
            let _ = worker.join();
            return Err(failure);
        }
        Err(_) => return join_interactive_worker(worker),
    };
    let platform_close_event = std::cell::RefCell::new(None);
    let worker_result = std::cell::RefCell::new(None);
    let shutdown_sender = work_sender.clone();
    let mut worker = Some(worker);
    let mut finalization_retries = 0_u64;
    let mut last_rendered_generation = (
        ready.initial_snapshot.snapshot_epoch,
        ready.initial_snapshot.snapshot_sequence,
    );

    let adapter = next_desktop_sdl_ash::run_interactive_with_shared_timed_frame_source_and_finalize(
        Arc::clone(&ready.initial_snapshot),
        &ready.render_content_catalog,
        &next_desktop_sdl_ash::DesktopRunOptions {
            maximum_frames,
            host_instance_id: ready.host_instance_id,
            resume_suspended_application: ready.resume_suspended_application,
            ..next_desktop_sdl_ash::DesktopRunOptions::default()
        },
        |events, elapsed| {
            if let Ok(failure) = failure_receiver.try_recv() {
                return Err(next_desktop_sdl_ash::DesktopAdapterError::client(
                    failure.code,
                    failure.message,
                ));
            }
            let scheduler_events = platform_events_before_close_boundary(
                events,
                &mut platform_close_event.borrow_mut(),
            );
            work_sender
                .send(InteractiveSimulationMessageV1::Advance {
                    elapsed,
                    events: scheduler_events,
                })
                .map_err(|_| {
                    let failure = failure_receiver.try_recv().unwrap_or_else(|_| {
                        AppFailure::cli(
                            "SESSION_RUNTIME_FAILED",
                            "simulation worker stopped before accepting a frame batch",
                        )
                    });
                    next_desktop_sdl_ash::DesktopAdapterError::client(failure.code, failure.message)
                })?;
            if let Ok(failure) = failure_receiver.try_recv() {
                return Err(next_desktop_sdl_ash::DesktopAdapterError::client(
                    failure.code,
                    failure.message,
                ));
            }

            let latest = latest_snapshot.read().map_err(|_| {
                next_desktop_sdl_ash::DesktopAdapterError::client(
                    "PLATFORM_PRESENTATION_STATE_POISONED",
                    "latest presentation snapshot lock was poisoned",
                )
            })?;
            let latest = latest.as_ref().ok_or_else(|| {
                next_desktop_sdl_ash::DesktopAdapterError::client(
                    "PLATFORM_PRESENTATION_SNAPSHOT_MISSING",
                    "simulation worker published no initial presentation snapshot",
                )
            })?;
            let generation = (latest.snapshot_epoch, latest.snapshot_sequence);
            if generation == last_rendered_generation {
                Ok(None)
            } else {
                last_rendered_generation = generation;
                Ok(Some(Arc::clone(latest)))
            }
        },
        || {
            let (completion_sender, completion_receiver) = mpsc::sync_channel(1);
            let shutdown = InteractiveSimulationMessageV1::Shutdown {
                platform_close_event: platform_close_event.borrow().clone().map(Box::new),
                rendered_objects: 0,
                completion_sender,
            };
            let completion = if worker.is_some() && shutdown_sender.send(shutdown).is_ok() {
                completion_receiver.recv().ok()
            } else {
                None
            };
            if completion.as_ref().is_some_and(|reply| reply.closed) {
                let joined = worker.take().map_or_else(
                    || {
                        Err(AppFailure::cli(
                            "SESSION_RUNTIME_FAILED",
                            "simulation worker was finalized more than once",
                        ))
                    },
                    join_interactive_worker,
                );
                *worker_result.borrow_mut() = Some(joined);
                return next_desktop_sdl_ash::DesktopApplicationFinalization::Complete;
            }

            finalization_retries = finalization_retries.saturating_add(1);
            if finalization_retries.is_power_of_two() {
                let message = completion
                    .as_ref()
                    .and_then(|reply| reply.result.as_ref().err())
                    .map_or(
                        "simulation worker did not confirm durable Closed",
                        |failure| failure.message.as_str(),
                    );
                eprintln!(
                    "next_game: keeping desktop adapter alive for exact close retry {}: {}",
                    finalization_retries, message
                );
            }
            next_desktop_sdl_ash::DesktopApplicationFinalization::Retry
        },
    );

    let mut worker_report = worker_result.into_inner().ok_or_else(|| {
        AppFailure::cli(
            "SESSION_RUNTIME_FAILED",
            "desktop adapter released without finalizing the simulation worker",
        )
    })??;
    let adapter =
        adapter.map_err(|error| AppFailure::cli(error.diagnostic_code(), error.to_string()))?;
    worker_report.interactive_host_object_count = adapter.rendered_objects;

    eprintln!(
        "next_game: desktop session closed: frames={}, platform_events={}, controls={}, resizes={}, focus_events={}, fullscreen={}, recoveries={}",
        adapter.rendered_frames,
        adapter.normalized_events,
        adapter.control_events,
        adapter.resize_events,
        adapter.focus_events,
        adapter.fullscreen_events,
        adapter.device_recoveries,
    );
    Ok(worker_report)
}

#[cfg(any(feature = "desktop-sdl-ash", test))]
const INTERACTIVE_SIMULATION_QUEUE_CAPACITY: usize = 8;

#[cfg(any(feature = "desktop-sdl-ash", test))]
enum InteractiveSimulationMessageV1 {
    Advance {
        elapsed: Duration,
        events: Vec<next_contracts::platform::PlatformEventV1>,
    },
    Shutdown {
        platform_close_event: Option<Box<next_contracts::platform::PlatformEventV1>>,
        rendered_objects: u64,
        completion_sender: SyncSender<InteractiveShutdownReplyV1>,
    },
}

#[cfg(any(feature = "desktop-sdl-ash", test))]
struct InteractiveShutdownReplyV1 {
    closed: bool,
    result: Result<RunReportV1, AppFailure>,
}

#[cfg(any(feature = "desktop-sdl-ash", test))]
enum InteractiveShutdownAttemptV1<T> {
    Retry(AppFailure),
    Closed(Result<T, AppFailure>),
}

#[cfg(any(feature = "desktop-sdl-ash", test))]
fn resolve_interactive_shutdown_attempt<T>(
    close_result: Result<T, AppFailure>,
    pending_failure: Option<&AppFailure>,
) -> InteractiveShutdownAttemptV1<T> {
    match close_result {
        Ok(value) => {
            InteractiveShutdownAttemptV1::Closed(pending_failure.cloned().map_or(Ok(value), Err))
        }
        Err(failure) => InteractiveShutdownAttemptV1::Retry(failure),
    }
}

#[cfg(any(feature = "desktop-sdl-ash", test))]
struct InteractiveReadyV1 {
    initial_snapshot: Arc<next_contracts::presentation::PresentationSnapshotV2>,
    render_content_catalog: next_contracts::render_content::RenderContentCatalogV1,
    host_instance_id: next_contracts::ids::PersistentId,
    resume_suspended_application: bool,
}

#[cfg(any(feature = "desktop-sdl-ash", test))]
fn run_interactive_simulation_session_worker(
    launch: LaunchRequestV1,
    capabilities: next_contracts::platform::PlatformCapabilitySetV1,
    work_receiver: Receiver<InteractiveSimulationMessageV1>,
    latest_snapshot: Arc<RwLock<Option<Arc<next_contracts::presentation::PresentationSnapshotV2>>>>,
    ready_sender: SyncSender<Result<InteractiveReadyV1, AppFailure>>,
    failure_sender: SyncSender<AppFailure>,
) -> Result<RunReportV1, AppFailure> {
    let prepared = prepare_interactive_worker(launch, &capabilities, &latest_snapshot);
    let (mut application, ready) = match prepared {
        Ok(prepared) => prepared,
        Err(failure) => {
            let _ = ready_sender.send(Err(failure.clone()));
            return Err(failure);
        }
    };
    if ready_sender.send(Ok(ready)).is_err() {
        return finish_interactive_worker(&mut application, None, 0).and(Err(AppFailure::cli(
            "SESSION_RUNTIME_FAILED",
            "interactive host stopped before receiving worker readiness",
        )));
    }

    let mut fixed_step = FixedStepLiveSchedulerV1::reference_game_v1();
    let mut pending_failure = None;
    loop {
        let message = match work_receiver.recv() {
            Ok(message) => message,
            Err(_) => {
                let failure = pending_failure.unwrap_or_else(|| {
                    AppFailure::cli(
                        "SESSION_RUNTIME_FAILED",
                        "interactive frame queue disconnected before shutdown",
                    )
                });
                return finish_interactive_worker(&mut application, None, 0).and(Err(failure));
            }
        };
        match message {
            InteractiveSimulationMessageV1::Advance { elapsed, events }
                if pending_failure.is_none() =>
            {
                match fixed_step
                    .advance_reference_game_presentation(&mut application, elapsed, &events)
                    .map_err(AppFailure::application)
                {
                    Ok(Some(next_snapshot)) => match latest_snapshot.write() {
                        Ok(mut latest) => *latest = Some(Arc::new(next_snapshot)),
                        Err(_) => record_interactive_worker_failure(
                            &mut pending_failure,
                            &failure_sender,
                            AppFailure::cli(
                                "PLATFORM_PRESENTATION_STATE_POISONED",
                                "latest presentation snapshot lock was poisoned",
                            ),
                        ),
                    },
                    Ok(None) => {}
                    Err(failure) => record_interactive_worker_failure(
                        &mut pending_failure,
                        &failure_sender,
                        failure,
                    ),
                }
            }
            InteractiveSimulationMessageV1::Advance { .. } => {}
            InteractiveSimulationMessageV1::Shutdown {
                platform_close_event,
                rendered_objects,
                completion_sender,
            } => {
                let report = finish_interactive_worker(
                    &mut application,
                    platform_close_event.as_deref(),
                    rendered_objects,
                );
                match resolve_interactive_shutdown_attempt(report, pending_failure.as_ref()) {
                    InteractiveShutdownAttemptV1::Closed(result) => {
                        let _ = completion_sender.send(InteractiveShutdownReplyV1 {
                            closed: true,
                            result: result.clone(),
                        });
                        return result;
                    }
                    InteractiveShutdownAttemptV1::Retry(failure) => {
                        let _ = completion_sender.send(InteractiveShutdownReplyV1 {
                            closed: false,
                            result: Err(failure),
                        });
                    }
                }
            }
        }
    }
}

#[cfg(any(feature = "desktop-sdl-ash", test))]
fn prepare_interactive_worker(
    launch: LaunchRequestV1,
    capabilities: &next_contracts::platform::PlatformCapabilitySetV1,
    latest_snapshot: &Arc<
        RwLock<Option<Arc<next_contracts::presentation::PresentationSnapshotV2>>>,
    >,
) -> Result<(ApplicationCoordinator, InteractiveReadyV1), AppFailure> {
    let mut application =
        ApplicationCoordinator::launch_or_resume(launch).map_err(AppFailure::application)?;
    eprintln!(
        "next_game: session {} active",
        application.state().session_id.to_hex()
    );
    let resume_suspended_application =
        application.state().state == next_contracts::session::ApplicationSessionStatusV1::Suspended;
    let run = begin_or_resume_reference_game_live(&mut application)?;
    let initial_snapshot = Arc::new(run.presentation_snapshot.ok_or_else(|| {
        AppFailure::cli(
            "PLATFORM_PRESENTATION_SNAPSHOT_MISSING",
            "interactive target produced no presentation snapshot",
        )
    })?);
    let render_content_catalog = application
        .activated_project()
        .render_content_catalog
        .clone();
    let host_instance_id = application
        .register_platform_host(capabilities)
        .map_err(AppFailure::application)?;
    *latest_snapshot.write().map_err(|_| {
        AppFailure::cli(
            "PLATFORM_PRESENTATION_STATE_POISONED",
            "latest presentation snapshot lock was poisoned",
        )
    })? = Some(Arc::clone(&initial_snapshot));
    Ok((
        application,
        InteractiveReadyV1 {
            initial_snapshot,
            render_content_catalog,
            host_instance_id,
            resume_suspended_application,
        },
    ))
}

#[cfg(any(feature = "desktop-sdl-ash", test))]
fn record_interactive_worker_failure(
    pending_failure: &mut Option<AppFailure>,
    failure_sender: &SyncSender<AppFailure>,
    failure: AppFailure,
) {
    if pending_failure.is_none() {
        let _ = failure_sender.try_send(failure.clone());
        *pending_failure = Some(failure);
    }
}

#[cfg(any(feature = "desktop-sdl-ash", test))]
fn finish_interactive_worker(
    application: &mut ApplicationCoordinator,
    platform_close_event: Option<&next_contracts::platform::PlatformEventV1>,
    rendered_objects: u64,
) -> Result<RunReportV1, AppFailure> {
    let run = application
        .current_live_run()
        .map_err(AppFailure::application)?;
    let close_options = next_application::CloseExecutionOptionsV1::default();
    let close = if let Some(event) = platform_close_event {
        application.close_from_platform_event(event, close_options)
    } else {
        application.close(close_options)
    }
    .map_err(AppFailure::application)?;
    if !matches!(close, ApplicationCloseOutcomeV1::Closed { .. }) {
        return Err(AppFailure::cli(
            "SESSION_FINAL_SAVE_FAILED",
            "application close did not reach a terminal receipt",
        ));
    }
    RunReportV1::new(CompositionRootV1::Game, &run, &close, rendered_objects).ok_or_else(|| {
        AppFailure::cli(
            "SESSION_TERMINAL_RECEIPT_MISSING",
            "closed application has no terminal receipt",
        )
    })
}

#[cfg(feature = "desktop-sdl-ash")]
fn join_interactive_worker(
    worker: std::thread::JoinHandle<Result<RunReportV1, AppFailure>>,
) -> Result<RunReportV1, AppFailure> {
    worker.join().map_err(|_| {
        AppFailure::cli(
            "SESSION_RUNTIME_FAILED",
            "simulation worker panicked during the interactive session",
        )
    })?
}

#[cfg(any(feature = "desktop-sdl-ash", test))]
fn platform_events_before_close_boundary(
    events: &[next_contracts::platform::PlatformEventV1],
    platform_close_event: &mut Option<next_contracts::platform::PlatformEventV1>,
) -> Vec<next_contracts::platform::PlatformEventV1> {
    if platform_close_event.is_none() {
        *platform_close_event = events
            .iter()
            .filter(|event| {
                event.kind == next_contracts::platform::PlatformEventKindV1::CloseRequested
            })
            .min_by(|left, right| {
                (
                    left.host_instance_id,
                    &left.source_class,
                    left.source_sequence,
                    left.platform_event_id,
                )
                    .cmp(&(
                        right.host_instance_id,
                        &right.source_class,
                        right.source_sequence,
                        right.platform_event_id,
                    ))
            })
            .cloned();
    }
    let Some(close) = platform_close_event.as_ref() else {
        return events.to_vec();
    };
    events
        .iter()
        .filter(|event| {
            event.kind != next_contracts::platform::PlatformEventKindV1::CloseRequested
                && !(event.host_instance_id == close.host_instance_id
                    && event.source_class == close.source_class
                    && event.source_sequence >= close.source_sequence)
        })
        .cloned()
        .collect()
}

#[cfg(not(feature = "desktop-sdl-ash"))]
fn run_interactive_session(
    _launch: LaunchRequestV1,
    _maximum_frames: Option<u64>,
) -> Result<RunReportV1, AppFailure> {
    Err(AppFailure::cli(
        "PLATFORM_INTERACTIVE_ADAPTER_UNAVAILABLE",
        "interactive desktop adapter is not enabled",
    ))
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock, mpsc};
    use std::time::Duration;

    use super::{
        AppFailure, GameOptions, INTERACTIVE_SIMULATION_QUEUE_CAPACITY,
        InteractiveShutdownAttemptV1, InteractiveSimulationMessageV1,
        begin_or_resume_reference_game_live, platform_events_before_close_boundary,
        resolve_interactive_shutdown_attempt, run_interactive_simulation_session_worker,
    };
    use next_application::{
        ApplicationCloseOutcomeV1, ApplicationCoordinator, CloseExecutionOptionsV1,
        FixedStepLiveSchedulerV1, LaunchRequestV1,
    };
    use next_contracts::ids::SchemaId;
    use next_contracts::platform::{PlatformEventKindV1, PlatformEventPayloadV1, PlatformEventV1};
    use next_contracts::session::{CompositionRootV1, PresentationTargetKindV1};

    #[test]
    fn bounded_interactive_mode_requires_a_positive_frame_limit() {
        let options = GameOptions::parse(
            ["--interactive", "--maximum-frames", "1"]
                .into_iter()
                .map(str::to_owned),
        )
        .expect("bounded interactive options");
        assert!(options.interactive);
        assert_eq!(options.maximum_frames, Some(1));
        assert!(
            GameOptions::parse(["--maximum-frames", "1"].into_iter().map(str::to_owned)).is_err()
        );
    }

    #[test]
    fn shutdown_failure_retries_before_a_later_closed_confirmation() {
        let publication_failure = AppFailure::cli(
            "SESSION_RUNTIME_FAILED",
            "injected durable publication fault",
        );
        assert!(matches!(
            resolve_interactive_shutdown_attempt::<u8>(Err(publication_failure), None),
            InteractiveShutdownAttemptV1::Retry(_)
        ));

        let closed = resolve_interactive_shutdown_attempt(Ok(7_u8), None);
        assert!(matches!(
            closed,
            InteractiveShutdownAttemptV1::Closed(Ok(7))
        ));

        let pending_runtime_failure =
            AppFailure::cli("SESSION_RUNTIME_FAILED", "prior simulation failure");
        let closed_with_pending_failure =
            resolve_interactive_shutdown_attempt(Ok(9_u8), Some(&pending_runtime_failure));
        assert!(matches!(
            closed_with_pending_failure,
            InteractiveShutdownAttemptV1::Closed(Err(_))
        ));
    }

    #[test]
    fn simulation_worker_preserves_fifo_fixed_step_roots_and_publishes_only_latest_snapshot() {
        let serial_root = unique_test_directory("serial-worker-reference");
        let mut serial = ApplicationCoordinator::launch(LaunchRequestV1::reference(
            serial_root.clone(),
            CompositionRootV1::Game,
            PresentationTargetKindV1::Interactive,
        ))
        .expect("serial launch");
        begin_or_resume_reference_game_live(&mut serial).expect("serial live run");
        let mut serial_scheduler = FixedStepLiveSchedulerV1::reference_game_v1();
        for _ in 0..4 {
            serial_scheduler
                .advance_reference_game_presentation(&mut serial, Duration::from_millis(34), &[])
                .expect("serial fixed step");
        }
        let serial_run = serial.current_live_run().expect("serial current run");
        serial
            .close(CloseExecutionOptionsV1::default())
            .expect("serial close");

        let worker_root = unique_test_directory("threaded-worker-reference");
        let launch = LaunchRequestV1::reference(
            worker_root.clone(),
            CompositionRootV1::Game,
            PresentationTargetKindV1::Interactive,
        );
        let capabilities = launch
            .platform_capability_set
            .clone()
            .expect("interactive capabilities");
        let latest = Arc::new(RwLock::new(None));
        let (work_sender, work_receiver) =
            mpsc::sync_channel(INTERACTIVE_SIMULATION_QUEUE_CAPACITY);
        let (ready_sender, ready_receiver) = mpsc::sync_channel(1);
        let (failure_sender, failure_receiver) = mpsc::sync_channel(1);
        let worker_latest = Arc::clone(&latest);
        let worker = std::thread::spawn(move || {
            run_interactive_simulation_session_worker(
                launch,
                capabilities,
                work_receiver,
                worker_latest,
                ready_sender,
                failure_sender,
            )
        });
        let ready = ready_receiver
            .recv()
            .expect("worker readiness channel")
            .expect("worker readiness");
        assert_eq!(ready.initial_snapshot.simulation_tick, 0);
        assert_ne!(
            ready.render_content_catalog.catalog_sha256(),
            next_contracts::ids::ContentHash::default()
        );
        assert_ne!(
            ready.host_instance_id,
            next_contracts::ids::PersistentId::default()
        );
        assert!(!ready.resume_suspended_application);
        for _ in 0..4 {
            work_sender
                .send(InteractiveSimulationMessageV1::Advance {
                    elapsed: Duration::from_millis(34),
                    events: Vec::new(),
                })
                .expect("bounded FIFO accepts fixed-step batch");
        }
        let (completion_sender, completion_receiver) = mpsc::sync_channel(1);
        work_sender
            .send(InteractiveSimulationMessageV1::Shutdown {
                platform_close_event: None,
                rendered_objects: 7,
                completion_sender,
            })
            .expect("ordered worker shutdown");
        let completion = completion_receiver
            .recv()
            .expect("worker shutdown completion");
        assert!(completion.closed);
        assert!(completion.result.is_ok());
        let threaded_report = worker
            .join()
            .expect("worker does not panic")
            .expect("worker session succeeds");
        assert!(failure_receiver.try_recv().is_err());
        let latest = latest
            .read()
            .expect("latest snapshot lock")
            .clone()
            .expect("latest snapshot");

        assert_eq!(latest.simulation_tick, 4);
        assert_eq!(threaded_report.ticks, serial_run.ticks);
        assert_eq!(
            threaded_report.authoritative_state_root,
            serial_run.authoritative_state_root.to_hex()
        );
        assert_eq!(
            threaded_report.command_archive_root,
            serial_run.command_archive_root.to_hex()
        );
        assert_eq!(
            threaded_report.command_identity_index_root,
            serial_run.command_identity_index_root.to_hex()
        );
        assert_eq!(threaded_report.interactive_host_object_count, 7);

        std::fs::remove_dir_all(serial_root).expect("remove serial state");
        std::fs::remove_dir_all(worker_root).expect("remove worker state");
    }

    #[test]
    fn desktop_launch_continues_an_existing_live_session_instead_of_beginning_twice() {
        let state_root = unique_test_directory("restart-live");
        let launch = LaunchRequestV1::reference(
            state_root.clone(),
            CompositionRootV1::Game,
            PresentationTargetKindV1::Interactive,
        );
        let mut first =
            ApplicationCoordinator::launch_or_resume(launch.clone()).expect("first launch");
        let initial =
            begin_or_resume_reference_game_live(&mut first).expect("begin first live run");
        let session_id = first.state().session_id;
        drop(first);

        let mut restarted =
            ApplicationCoordinator::launch_or_resume(launch).expect("restart existing live run");
        let resumed =
            begin_or_resume_reference_game_live(&mut restarted).expect("continue recovered run");

        assert_eq!(restarted.state().session_id, session_id);
        assert_eq!(resumed.session_id, initial.session_id);
        assert_eq!(
            resumed.project_composition_lock_hash,
            initial.project_composition_lock_hash
        );
        assert_eq!(resumed.ticks, initial.ticks);
        assert_eq!(resumed.events, initial.events);
        assert_eq!(resumed.rpg_events, initial.rpg_events);
        assert_eq!(
            resumed.authoritative_revision,
            initial.authoritative_revision
        );
        assert_eq!(
            resumed.authoritative_state_root,
            initial.authoritative_state_root
        );
        assert_eq!(resumed.command_archive_root, initial.command_archive_root);
        assert_eq!(
            resumed.command_identity_index_root,
            initial.command_identity_index_root
        );
        assert_eq!(resumed.command_ledger_hash, initial.command_ledger_hash);
        assert_eq!(
            resumed.presentation_input_count,
            initial.presentation_input_count
        );
        let initial_presentation = initial
            .presentation_snapshot
            .expect("initial presentation snapshot");
        let resumed_presentation = resumed
            .presentation_snapshot
            .expect("recovery-cut presentation snapshot");
        assert_ne!(
            resumed_presentation.snapshot_epoch,
            initial_presentation.snapshot_epoch
        );
        assert_eq!(resumed_presentation.snapshot_sequence, 0);
        assert!(
            resumed_presentation
                .camera_records()
                .all(|camera| camera.cut)
        );
        std::fs::remove_dir_all(state_root).expect("remove test state");
    }

    #[test]
    fn close_boundary_does_not_admit_same_source_suffix_before_close_transaction() {
        let state_root = unique_test_directory("close-boundary");
        let launch = LaunchRequestV1::reference(
            state_root.clone(),
            CompositionRootV1::Game,
            PresentationTargetKindV1::Interactive,
        );
        let capabilities = launch
            .platform_capability_set
            .clone()
            .expect("interactive capabilities");
        let mut application = ApplicationCoordinator::launch(launch).expect("launch");
        application
            .begin_reference_game_live(true)
            .expect("begin live game");
        let host_instance_id = application
            .register_platform_host(&capabilities)
            .expect("register platform host");
        let source_class =
            SchemaId::new("nextengine.platform.source.window-test").expect("source class");
        let event = |sequence, kind, payload| {
            PlatformEventV1::new(
                host_instance_id,
                source_class.clone(),
                sequence,
                sequence,
                kind,
                payload,
                capabilities.canonical_hash,
            )
            .expect("platform event")
        };
        let before = event(
            0,
            PlatformEventKindV1::FocusChanged,
            PlatformEventPayloadV1::FocusChanged { focused: false },
        );
        let close = event(
            1,
            PlatformEventKindV1::CloseRequested,
            PlatformEventPayloadV1::Reason {
                reason: SchemaId::new("nextengine.platform.reason.window-close")
                    .expect("close reason"),
            },
        );
        let after = event(
            2,
            PlatformEventKindV1::FocusChanged,
            PlatformEventPayloadV1::FocusChanged { focused: true },
        );
        let mut selected_close = None;
        let scheduler_events = platform_events_before_close_boundary(
            &[before.clone(), close.clone(), after],
            &mut selected_close,
        );
        assert_eq!(scheduler_events, vec![before]);
        assert_eq!(selected_close, Some(close.clone()));

        let mut scheduler = FixedStepLiveSchedulerV1::reference_game_v1();
        scheduler
            .advance_reference_game(&mut application, Duration::ZERO, &scheduler_events)
            .expect("admit only the pre-close source prefix");
        let closed = application
            .close_from_platform_event(&close, CloseExecutionOptionsV1::default())
            .expect("close remains the exact next source event");
        assert!(matches!(closed, ApplicationCloseOutcomeV1::Closed { .. }));
        std::fs::remove_dir_all(state_root).expect("remove test state");
    }

    fn unique_test_directory(label: &str) -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::time::{SystemTime, UNIX_EPOCH};

        static SEQUENCE: AtomicU64 = AtomicU64::new(0);

        std::env::temp_dir().join(format!(
            "nextengine-game-{label}-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock after epoch")
                .as_nanos(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ))
    }
}
