#![forbid(unsafe_code)]

use std::path::PathBuf;

use next_application::{
    ApplicationCloseOutcomeV2, ApplicationCoordinator, ApplicationError, DiagnosticContextV1,
    DiagnosticReportV1, LaunchRequestV1, ProjectSelectionV1, RunReportV1, default_user_state_root,
};
use next_contracts::ids::ContentHash;
use next_contracts::session::{CompositionRootV1, PresentationTargetKindV1};

mod motor_lab;

fn main() {
    // Developer profiling only: with `profile-tracy` enabled the Tracy client
    // starts in on-demand mode and activates when a Tracy profiler connects.
    #[cfg(feature = "profile-tracy")]
    let _tracy_client = tracy_client::Client::start();
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    if arguments.first().is_some_and(|value| value == "motor-lab") {
        std::process::exit(motor_lab::run(&arguments[1..]));
    }
    match run(arguments.into_iter()) {
        Ok(report) => println!("{}", report.to_json().expect("run report serializes")),
        Err(error) => {
            eprintln!("next_headless: {}", error.message);
            let report =
                DiagnosticReportV1::new(error.code, DiagnosticContextV1::message(&error.message));
            println!("{}", report.to_json().expect("diagnostic serializes"));
            std::process::exit(error.exit_code);
        }
    }
}

fn run(arguments: impl Iterator<Item = String>) -> Result<RunReportV1, AppFailure> {
    let options = HeadlessOptions::parse(arguments)?;
    if options.help {
        eprintln!(
            "usage: next_headless [--live-ticks <nonnegative-integer>] [--project <cooked-store>] [--lock <sha256>] [--state-root <directory>]\n       next_headless motor-lab"
        );
        return Err(AppFailure::help());
    }
    let state_root = options
        .state_root
        .unwrap_or(default_user_state_root("headless").map_err(AppFailure::application)?);
    let launch = LaunchRequestV1 {
        project: options.project.map_or(
            ProjectSelectionV1::Reference,
            ProjectSelectionV1::PublishedStateRoot,
        ),
        expected_project_lock: options.expected_lock,
        state_root,
        composition_root: CompositionRootV1::Headless,
        presentation_target: PresentationTargetKindV1::None,
        platform_capability_set: None,
    };
    let mut application =
        ApplicationCoordinator::launch_or_resume(launch).map_err(AppFailure::application)?;
    eprintln!(
        "next_headless: session {} active",
        application.state().session_id.to_hex()
    );
    let run = if let Some(live_ticks) = options.live_ticks {
        let mut run = match application.current_live_run() {
            Ok(run) => run,
            Err(ApplicationError::NoLiveRun) => application
                .begin_reference_game_live(true)
                .map_err(AppFailure::application)?,
            Err(error) => return Err(AppFailure::application(error)),
        };
        for _ in 0..live_ticks {
            run = application
                .advance_reference_game_live(&[])
                .map_err(AppFailure::application)?;
        }
        run
    } else {
        application
            .run_reference_game(true)
            .map_err(AppFailure::application)?
    };
    let close = application.close().map_err(AppFailure::application)?;
    if !matches!(close, ApplicationCloseOutcomeV2::Closed { .. }) {
        return Err(AppFailure::cli(
            "SESSION_FINAL_SAVE_FAILED",
            "application close did not reach a terminal receipt",
        ));
    }
    RunReportV1::new(CompositionRootV1::Headless, &run, &close, 0).ok_or_else(|| {
        AppFailure::cli(
            "SESSION_TERMINAL_RECEIPT_MISSING",
            "closed application has no terminal receipt",
        )
    })
}

#[derive(Debug, Default)]
struct HeadlessOptions {
    live_ticks: Option<u64>,
    project: Option<PathBuf>,
    expected_lock: Option<ContentHash>,
    state_root: Option<PathBuf>,
    help: bool,
}

impl HeadlessOptions {
    fn parse(mut arguments: impl Iterator<Item = String>) -> Result<Self, AppFailure> {
        let mut options = Self::default();
        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--live-ticks" => {
                    let value = required_value(&mut arguments, "--live-ticks")?;
                    let ticks = value.parse::<u64>().map_err(|_| {
                        AppFailure::argument("--live-ticks requires a nonnegative integer")
                    })?;
                    if options.live_ticks.replace(ticks).is_some() {
                        return Err(AppFailure::argument("--live-ticks specified twice"));
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
        Ok(options)
    }
}

#[cfg(test)]
mod tests {
    use super::HeadlessOptions;

    #[test]
    fn live_tick_mode_accepts_zero_and_rejects_invalid_or_duplicate_counts() {
        let zero = HeadlessOptions::parse(["--live-ticks", "0"].into_iter().map(str::to_owned))
            .expect("zero-tick live smoke");
        assert_eq!(zero.live_ticks, Some(0));
        assert!(
            HeadlessOptions::parse(["--live-ticks", "-1"].into_iter().map(str::to_owned)).is_err()
        );
        assert!(
            HeadlessOptions::parse(
                ["--live-ticks", "0", "--live-ticks", "1"]
                    .into_iter()
                    .map(str::to_owned)
            )
            .is_err()
        );
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
