#![forbid(unsafe_code)]

use std::path::PathBuf;

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
    let launch = LaunchRequestV1 {
        project: options.project.map_or(
            ProjectSelectionV1::Reference,
            ProjectSelectionV1::PublishedStateRoot,
        ),
        expected_project_lock: options.expected_lock,
        state_root,
        composition_root: CompositionRootV1::Game,
        presentation_target: target,
    };
    let mut application =
        ApplicationCoordinator::launch_or_resume(launch).map_err(AppFailure::application)?;
    eprintln!(
        "next_game: session {} active",
        application.state().session_id.to_hex()
    );
    let run = application
        .run_reference_game(true)
        .map_err(AppFailure::application)?;
    let render_content_catalog = application
        .activated_project()
        .render_content_catalog
        .clone();
    let adapter = if options.interactive {
        let snapshot = run.presentation_snapshot.as_ref().ok_or_else(|| {
            AppFailure::cli(
                "PLATFORM_PRESENTATION_SNAPSHOT_MISSING",
                "interactive target produced no presentation snapshot",
            )
        })?;
        run_interactive(snapshot, &render_content_catalog, options.maximum_frames)
    } else {
        Ok(0)
    };
    let close = application
        .close(next_application::CloseExecutionOptionsV1::default())
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
    snapshot: &next_contracts::presentation::PresentationSnapshotV2,
    render_content_catalog: &next_contracts::render_content::RenderContentCatalogV1,
    maximum_frames: Option<u64>,
) -> Result<u64, AppFailure> {
    let report = next_desktop_sdl_ash::run_interactive(
        snapshot,
        render_content_catalog,
        &next_desktop_sdl_ash::DesktopRunOptions {
            maximum_frames,
            ..next_desktop_sdl_ash::DesktopRunOptions::default()
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

#[cfg(not(feature = "desktop-sdl-ash"))]
fn run_interactive(
    _snapshot: &next_contracts::presentation::PresentationSnapshotV2,
    _render_content_catalog: &next_contracts::render_content::RenderContentCatalogV1,
    _maximum_frames: Option<u64>,
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
    use super::GameOptions;

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
}
