use std::path::PathBuf;

use next_application::ApplicationError;
use next_contracts::ids::ContentHash;

#[derive(Debug, Default)]
pub(super) struct GameOptions {
    pub(super) interactive: bool,
    pub(super) maximum_frames: Option<u64>,
    /// Plan `continuum-water/09` diagnostic capture: the rendered frame
    /// index to copy back and the PNG path to write it to.
    pub(super) capture_frame: Option<u64>,
    pub(super) capture_png: Option<PathBuf>,
    /// Plan `continuum-water/18`: `--capture-buffer` source name.
    pub(super) capture_buffer: Option<String>,
    /// Plan 18: `--capture-frames`, consecutive frames to capture.
    pub(super) capture_frames: Option<u32>,
    /// Plan `continuum-water/18`: sub-pixel projection jitter.
    pub(super) projection_jitter: bool,
    /// Plan `continuum-water/11` diagnostic: walk to the basin and turn the
    /// camera to it through scripted input at start.
    pub(super) start_at_water: bool,
    pub(super) project: Option<PathBuf>,
    pub(super) expected_lock: Option<ContentHash>,
    pub(super) state_root: Option<PathBuf>,
    pub(super) help: bool,
}

impl GameOptions {
    pub(super) fn parse(mut arguments: impl Iterator<Item = String>) -> Result<Self, AppFailure> {
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
                "--start-at-water" => {
                    if std::mem::replace(&mut options.start_at_water, true) {
                        return Err(AppFailure::argument("--start-at-water specified twice"));
                    }
                }
                "--capture-frame" => {
                    let value = required_value(&mut arguments, "--capture-frame")?;
                    let index = value.parse::<u64>().map_err(|_| {
                        AppFailure::argument("--capture-frame requires a non-negative integer")
                    })?;
                    if options.capture_frame.replace(index).is_some() {
                        return Err(AppFailure::argument("--capture-frame specified twice"));
                    }
                }
                "--capture-buffer" => {
                    let value = required_value(&mut arguments, "--capture-buffer")?;
                    if !matches!(
                        value.as_str(),
                        "color" | "scene" | "albedo" | "normal" | "motion" | "depth"
                    ) {
                        return Err(AppFailure::argument(
                            "--capture-buffer must be one of color, scene, albedo, normal, motion, depth",
                        ));
                    }
                    if options.capture_buffer.replace(value).is_some() {
                        return Err(AppFailure::argument("--capture-buffer specified twice"));
                    }
                }
                "--capture-frames" => {
                    let value = required_value(&mut arguments, "--capture-frames")?;
                    let count = value.parse::<u32>().map_err(|_| {
                        AppFailure::argument("--capture-frames requires a positive integer")
                    })?;
                    if count == 0 || options.capture_frames.replace(count).is_some() {
                        return Err(AppFailure::argument(
                            "--capture-frames must be one positive integer",
                        ));
                    }
                }
                "--projection-jitter" => {
                    if std::mem::replace(&mut options.projection_jitter, true) {
                        return Err(AppFailure::argument("--projection-jitter specified twice"));
                    }
                }
                "--capture-png" => {
                    let value = required_value(&mut arguments, "--capture-png")?;
                    if options.capture_png.replace(value.into()).is_some() {
                        return Err(AppFailure::argument("--capture-png specified twice"));
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
        if options.start_at_water && !options.interactive {
            return Err(AppFailure::argument(
                "--start-at-water requires --interactive",
            ));
        }
        if options.projection_jitter && !options.interactive {
            return Err(AppFailure::argument(
                "--projection-jitter requires --interactive",
            ));
        }
        if (options.capture_buffer.is_some() || options.capture_frames.is_some())
            && options.capture_frame.is_none()
        {
            return Err(AppFailure::argument(
                "--capture-buffer and --capture-frames require --capture-frame",
            ));
        }
        match (options.capture_frame, options.capture_png.as_ref()) {
            (None, None) => {}
            (Some(_), Some(_)) if options.interactive => {}
            (Some(_), Some(_)) => {
                return Err(AppFailure::argument(
                    "--capture-frame and --capture-png require --interactive",
                ));
            }
            _ => {
                return Err(AppFailure::argument(
                    "--capture-frame and --capture-png must be given together",
                ));
            }
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

#[derive(Debug, Clone)]
pub(super) struct AppFailure {
    pub(super) code: &'static str,
    pub(super) message: String,
    pub(super) exit_code: i32,
}

impl AppFailure {
    pub(super) fn cli(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            exit_code: 2,
        }
    }

    fn argument(message: impl Into<String>) -> Self {
        Self::cli("CLI_ARGUMENT_INVALID", message)
    }

    pub(super) fn application(error: ApplicationError) -> Self {
        Self {
            code: error.diagnostic_code(),
            message: error.to_string(),
            exit_code: 1,
        }
    }

    #[cfg(feature = "desktop-sdl-ash")]
    pub(super) fn interactive_worker(
        failure: next_application::InteractiveWorkerFailureV1,
    ) -> Self {
        Self {
            code: failure.code,
            message: failure.message,
            exit_code: failure.exit_code,
        }
    }

    pub(super) fn help() -> Self {
        Self {
            code: "CLI_HELP_REQUESTED",
            message: "help requested".to_owned(),
            exit_code: 0,
        }
    }
}
