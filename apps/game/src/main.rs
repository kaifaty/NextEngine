#![forbid(unsafe_code)]

use std::path::PathBuf;

#[cfg(feature = "desktop-sdl-ash")]
use next_application::FixedStepLiveSchedulerV1;
use next_application::{
    ApplicationCloseOutcomeV1, ApplicationCoordinator, ApplicationError, DiagnosticContextV1,
    DiagnosticReportV1, LaunchRequestV1, ProjectSelectionV1, RunReportV1, default_user_state_root,
};
use next_contracts::ids::ContentHash;
use next_contracts::session::{CompositionRootV1, PresentationTargetKindV1};

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
    let mut application =
        ApplicationCoordinator::launch_or_resume(launch).map_err(AppFailure::application)?;
    eprintln!(
        "next_game: session {} active",
        application.state().session_id.to_hex()
    );
    let mut run = if options.interactive {
        begin_or_resume_reference_game_live(&mut application)?
    } else {
        application
            .run_reference_game(true)
            .map_err(AppFailure::application)?
    };
    let render_content_catalog = application
        .activated_project()
        .render_content_catalog
        .clone();
    let mut platform_close_event = None;
    let adapter = if options.interactive {
        let snapshot = run.presentation_snapshot.as_ref().ok_or_else(|| {
            AppFailure::cli(
                "PLATFORM_PRESENTATION_SNAPSHOT_MISSING",
                "interactive target produced no presentation snapshot",
            )
        })?;
        run_interactive(
            &mut application,
            snapshot,
            &render_content_catalog,
            options.maximum_frames,
            &mut platform_close_event,
        )
    } else {
        Ok(0)
    };
    if options.interactive {
        run = application
            .current_live_run()
            .map_err(AppFailure::application)?;
    }
    let close_options = next_application::CloseExecutionOptionsV1::default();
    #[cfg(feature = "desktop-sdl-ash")]
    let close = if let Some(event) = platform_close_event.as_ref() {
        application.close_from_platform_event(event, close_options)
    } else {
        application.close(close_options)
    }
    .map_err(AppFailure::application)?;
    #[cfg(not(feature = "desktop-sdl-ash"))]
    let close = application
        .close(close_options)
        .map_err(AppFailure::application)?;
    if !matches!(close, ApplicationCloseOutcomeV1::Closed { .. }) {
        return Err(AppFailure::cli(
            "SESSION_FINAL_SAVE_FAILED",
            "application close did not reach a terminal receipt",
        ));
    }
    let interactive_host_object_count = adapter?;
    RunReportV1::new(
        CompositionRootV1::Game,
        &run,
        &close,
        interactive_host_object_count,
    )
    .ok_or_else(|| {
        AppFailure::cli(
            "SESSION_TERMINAL_RECEIPT_MISSING",
            "closed application has no terminal receipt",
        )
    })
}

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

#[derive(Debug, Default)]
struct GameOptions {
    interactive: bool,
    maximum_frames: Option<u64>,
    project: Option<PathBuf>,
    expected_lock: Option<ContentHash>,
    state_root: Option<PathBuf>,
    help: bool,
}

impl GameOptions {
    fn parse(mut arguments: impl Iterator<Item = String>) -> Result<Self, AppFailure> {
        let mut options = Self::default();
        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--interactive" => {
                    if std::mem::replace(&mut options.interactive, true) {
                        return Err(AppFailure::argument("--interactive specified twice"));
                    }
                }
                "--maximum-frames" => {
                    let value = required_value(&mut arguments, "--maximum-frames")?;
                    let maximum = value.parse::<u64>().map_err(|_| {
                        AppFailure::argument("--maximum-frames requires a positive integer")
                    })?;
                    if maximum == 0 || options.maximum_frames.replace(maximum).is_some() {
                        return Err(AppFailure::argument(
                            "--maximum-frames must be one positive integer",
                        ));
                    }
                }
                "--project" => {
                    let value = required_value(&mut arguments, "--project")?;
                    if options.project.replace(value.into()).is_some() {
                        return Err(AppFailure::argument("--project specified twice"));
                    }
                }
                "--lock" => {
                    let value = required_value(&mut arguments, "--lock")?;
                    let hash = parse_hash(&value)?;
                    if options.expected_lock.replace(hash).is_some() {
                        return Err(AppFailure::argument("--lock specified twice"));
                    }
                }
                "--state-root" => {
                    let value = required_value(&mut arguments, "--state-root")?;
                    if options.state_root.replace(value.into()).is_some() {
                        return Err(AppFailure::argument("--state-root specified twice"));
                    }
                }
                "--help" | "-h" => options.help = true,
                _ => {
                    return Err(AppFailure::argument(format!(
                        "unsupported argument: {argument}"
                    )));
                }
            }
        }
        if options.project.is_none() && options.expected_lock.is_some() {
            return Err(AppFailure::argument("--lock requires --project"));
        }
        if options.maximum_frames.is_some() && !options.interactive {
            return Err(AppFailure::argument(
                "--maximum-frames requires --interactive",
            ));
        }
        Ok(options)
    }
}

fn required_value(
    arguments: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<String, AppFailure> {
    arguments
        .next()
        .ok_or_else(|| AppFailure::argument(format!("{flag} requires a value")))
}

fn parse_hash(value: &str) -> Result<ContentHash, AppFailure> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(AppFailure::argument(
            "--lock must be a 64-character lowercase hex digest",
        ));
    }
    let mut bytes = [0_u8; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        bytes[index] = (hex_nibble(pair[0]) << 4) | hex_nibble(pair[1]);
    }
    Ok(ContentHash::from_bytes(bytes))
}

const fn hex_nibble(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        _ => 0,
    }
}

#[cfg(feature = "desktop-sdl-ash")]
fn run_interactive(
    application: &mut ApplicationCoordinator,
    snapshot: &next_contracts::presentation::PresentationSnapshotV2,
    render_content_catalog: &next_contracts::render_content::RenderContentCatalogV1,
    maximum_frames: Option<u64>,
    platform_close_event: &mut Option<next_contracts::platform::PlatformEventV1>,
) -> Result<u64, AppFailure> {
    let mut fixed_step = FixedStepLiveSchedulerV1::reference_game_v1();
    let capabilities = next_desktop_sdl_ash::desktop_capability_set()
        .map_err(|error| AppFailure::cli(error.diagnostic_code(), error.to_string()))?;
    let host_instance_id = application
        .register_platform_host(&capabilities)
        .map_err(AppFailure::application)?;
    let resume_suspended_application =
        application.state().state == next_contracts::session::ApplicationSessionStatusV1::Suspended;
    let report = next_desktop_sdl_ash::run_interactive_with_timed_frame_source(
        snapshot,
        render_content_catalog,
        &next_desktop_sdl_ash::DesktopRunOptions {
            maximum_frames,
            host_instance_id,
            resume_suspended_application,
            ..next_desktop_sdl_ash::DesktopRunOptions::default()
        },
        |events, elapsed| {
            let scheduler_events =
                platform_events_before_close_boundary(events, platform_close_event);
            let run = fixed_step
                .advance_reference_game_presentation(application, elapsed, &scheduler_events)
                .map_err(|error| {
                    next_desktop_sdl_ash::DesktopAdapterError::client(
                        error.diagnostic_code(),
                        error.to_string(),
                    )
                })?;
            Ok(run)
        },
    )
    .map_err(|error| AppFailure::cli(error.diagnostic_code(), error.to_string()))?;
    eprintln!(
        "next_game: desktop session closed: frames={}, platform_events={}, controls={}, resizes={}, focus_events={}, fullscreen={}, recoveries={}",
        report.rendered_frames,
        report.normalized_events,
        report.control_events,
        report.resize_events,
        report.focus_events,
        report.fullscreen_events,
        report.device_recoveries,
    );
    Ok(report.rendered_objects)
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
fn run_interactive(
    _application: &mut ApplicationCoordinator,
    _snapshot: &next_contracts::presentation::PresentationSnapshotV2,
    _render_content_catalog: &next_contracts::render_content::RenderContentCatalogV1,
    _maximum_frames: Option<u64>,
    _platform_close_event: &mut Option<next_contracts::platform::PlatformEventV1>,
) -> Result<u64, AppFailure> {
    Err(AppFailure::cli(
        "PLATFORM_INTERACTIVE_ADAPTER_UNAVAILABLE",
        "interactive desktop adapter is not enabled",
    ))
}

#[derive(Debug)]
struct AppFailure {
    code: &'static str,
    message: String,
    exit_code: i32,
}

impl AppFailure {
    fn cli(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            exit_code: 2,
        }
    }

    fn argument(message: impl Into<String>) -> Self {
        Self::cli("CLI_ARGUMENT_INVALID", message)
    }

    fn application(error: ApplicationError) -> Self {
        Self {
            code: error.diagnostic_code(),
            message: error.to_string(),
            exit_code: 1,
        }
    }

    fn help() -> Self {
        Self {
            code: "CLI_HELP_REQUESTED",
            message: "help requested".to_owned(),
            exit_code: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{
        GameOptions, begin_or_resume_reference_game_live, platform_events_before_close_boundary,
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
