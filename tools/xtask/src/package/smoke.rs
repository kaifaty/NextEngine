use std::env;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use super::{bounded_text, package_error, trim_ascii_whitespace, validate_hash};

const MAX_SMOKE_STDOUT_BYTES: usize = 1024 * 1024;
const MAX_SMOKE_STDERR_BYTES: usize = 64 * 1024;
const PACKAGE_SMOKE_TIMEOUT: Duration = Duration::from_secs(30);
const PACKAGE_SMOKE_POLL_INTERVAL: Duration = Duration::from_millis(10);
pub(super) const LINUX_DYNAMIC_LOADER_FAILURE_EXIT_CODE: i32 = 127;
pub(super) const WINDOWS_STATUS_INVALID_IMAGE_FORMAT: i32 = 0xC000_007B_u32 as i32;
pub(super) const WINDOWS_STATUS_INVALID_IMAGE_LE_FORMAT: i32 = 0xC000_012E_u32 as i32;
pub(super) const WINDOWS_STATUS_INVALID_IMAGE_NOT_MZ: i32 = 0xC000_012F_u32 as i32;
pub(super) const WINDOWS_STATUS_INVALID_IMAGE_WIN_16: i32 = 0xC000_0131_u32 as i32;
pub(super) const WINDOWS_STATUS_DLL_NOT_FOUND: i32 = 0xC000_0135_u32 as i32;
pub(super) const WINDOWS_STATUS_ORDINAL_NOT_FOUND: i32 = 0xC000_0138_u32 as i32;
pub(super) const WINDOWS_STATUS_ENTRYPOINT_NOT_FOUND: i32 = 0xC000_0139_u32 as i32;

pub(super) fn run_packaged_binary(
    binary: &Path,
    arguments: &[&str],
    smoke_session_root: &Path,
    package_root: &Path,
    expected_composition_root: &str,
    expected_project_lock: &str,
) -> Result<next_application::RunReportV1, String> {
    run_packaged_binary_with_timeout(
        binary,
        arguments,
        smoke_session_root,
        package_root,
        expected_composition_root,
        expected_project_lock,
        PACKAGE_SMOKE_TIMEOUT,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn run_packaged_binary_with_timeout(
    binary: &Path,
    arguments: &[&str],
    smoke_session_root: &Path,
    package_root: &Path,
    expected_composition_root: &str,
    expected_project_lock: &str,
    timeout: Duration,
) -> Result<next_application::RunReportV1, String> {
    let state_root = smoke_session_root.join("state");
    let home = smoke_session_root.join("home");
    let local_app_data = smoke_session_root.join("local-app-data");
    let xdg_state_home = smoke_session_root.join("xdg-state");
    let roaming_app_data = smoke_session_root.join("roaming-app-data");
    let program_data = smoke_session_root.join("program-data");
    let temporary = smoke_session_root.join("temp");
    for directory in [
        &state_root,
        &home,
        &local_app_data,
        &xdg_state_home,
        &roaming_app_data,
        &program_data,
        &temporary,
    ] {
        fs::create_dir_all(directory).map_err(|error| {
            format!(
                "NATIVE_GATE_PACKAGE_INVALID: failed to prepare smoke directory {}: {error}",
                directory.display()
            )
        })?;
    }
    let mut command = Command::new(binary);
    command
        .args(arguments)
        .arg("--state-root")
        .arg(&state_root)
        .current_dir(package_root);
    configure_smoke_environment(
        &mut command,
        &home,
        &local_app_data,
        &roaming_app_data,
        &xdg_state_home,
        &program_data,
        &temporary,
    );
    let output = run_bounded_command(command, binary, timeout)?;
    if !output.status.success() {
        if let Some(code) =
            runtime_prerequisite_failure_code(output.status.code(), &output.stdout, &output.stderr)
        {
            return Err(format!(
                "{code}: {} smoke failed with {}: {}{}",
                binary.display(),
                output.status,
                bounded_text(&output.stderr),
                if output.stderr_truncated {
                    " [stderr truncated]"
                } else {
                    ""
                }
            ));
        }
        return package_error(format!(
            "{} smoke failed with {}: {}{}",
            binary.display(),
            output.status,
            bounded_text(&output.stderr),
            if output.stderr_truncated {
                " [stderr truncated]"
            } else {
                ""
            }
        ));
    }
    if output.stdout_truncated {
        return package_error(format!("{} smoke report exceeds 1 MiB", binary.display()));
    }
    let report: next_application::RunReportV1 =
        serde_json::from_slice(trim_ascii_whitespace(&output.stdout)).map_err(|error| {
            format!(
                "NATIVE_GATE_PACKAGE_INVALID: {} emitted invalid run report JSON: {error}",
                binary.display()
            )
        })?;
    if report.schema_version != 1
        || report.status != "PASS"
        || report.composition_root != expected_composition_root
        || report.project_composition_lock_hash != expected_project_lock
        || report.close_result != "Saved"
    {
        return package_error(format!(
            "{} did not report a successful exact-project {} run",
            binary.display(),
            expected_composition_root
        ));
    }
    validate_hash(
        "run authoritative state root",
        &report.authoritative_state_root,
    )?;
    validate_hash("run command ledger hash", &report.command_ledger_hash)?;
    Ok(report)
}

pub(super) fn runtime_prerequisite_failure_code(
    exit_code: Option<i32>,
    stdout: &[u8],
    stderr: &[u8],
) -> Option<&'static str> {
    let stdout = String::from_utf8_lossy(stdout);
    let stderr = String::from_utf8_lossy(stderr);
    let contains = |code: &str| stdout.contains(code) || stderr.contains(code);
    if [
        "PLATFORM_DESKTOP_RUNTIME_UNAVAILABLE",
        "PLATFORM_GRAPHICS_LOADER_UNAVAILABLE",
        "PLATFORM_GRAPHICS_ICD_UNAVAILABLE",
    ]
    .into_iter()
    .any(contains)
    {
        Some("NATIVE_GATE_PACKAGE_RUNTIME_DEPENDENCY_MISSING")
    } else if contains("PLATFORM_GRAPHICS_LOADER_VERSION_UNSUPPORTED") {
        Some("NATIVE_GATE_PACKAGE_RUNTIME_ABI_UNSUPPORTED")
    } else if (exit_code == Some(LINUX_DYNAMIC_LOADER_FAILURE_EXIT_CODE)
        && contains("error while loading shared libraries"))
        || exit_code == Some(WINDOWS_STATUS_DLL_NOT_FOUND)
    {
        Some("NATIVE_GATE_PACKAGE_RUNTIME_DEPENDENCY_MISSING")
    } else if exit_code.is_some_and(|code| {
        matches!(
            code,
            WINDOWS_STATUS_INVALID_IMAGE_FORMAT
                | WINDOWS_STATUS_INVALID_IMAGE_LE_FORMAT
                | WINDOWS_STATUS_INVALID_IMAGE_NOT_MZ
                | WINDOWS_STATUS_INVALID_IMAGE_WIN_16
                | WINDOWS_STATUS_ORDINAL_NOT_FOUND
                | WINDOWS_STATUS_ENTRYPOINT_NOT_FOUND
        )
    }) {
        Some("NATIVE_GATE_PACKAGE_RUNTIME_ABI_UNSUPPORTED")
    } else {
        None
    }
}

pub(super) fn configure_smoke_environment(
    command: &mut Command,
    home: &Path,
    local_app_data: &Path,
    roaming_app_data: &Path,
    xdg_state_home: &Path,
    program_data: &Path,
    temporary: &Path,
) {
    command.env_clear();
    command
        .env("HOME", home)
        .env("USERPROFILE", home)
        .env("LOCALAPPDATA", local_app_data)
        .env("APPDATA", roaming_app_data)
        .env("XDG_STATE_HOME", xdg_state_home)
        .env("PROGRAMDATA", program_data)
        .env("ALLUSERSPROFILE", program_data)
        .env("TMP", temporary)
        .env("TEMP", temporary)
        .env("TMPDIR", temporary);

    #[cfg(windows)]
    for name in ["SYSTEMROOT", "WINDIR"] {
        copy_environment_if_present(command, name);
    }
    #[cfg(target_os = "linux")]
    for name in [
        "DISPLAY",
        "WAYLAND_DISPLAY",
        "XDG_RUNTIME_DIR",
        "DBUS_SESSION_BUS_ADDRESS",
        "XAUTHORITY",
    ] {
        copy_environment_if_present(command, name);
    }
}

fn copy_environment_if_present(command: &mut Command, name: &str) {
    if let Some(value) = env::var_os(name) {
        command.env(name, value);
    }
}

struct BoundedCommandOutput {
    status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    stdout_truncated: bool,
    stderr_truncated: bool,
}

fn run_bounded_command(
    mut command: Command,
    binary: &Path,
    timeout: Duration,
) -> Result<BoundedCommandOutput, String> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn().map_err(|error| {
        format!(
            "NATIVE_GATE_PACKAGE_RUNTIME_DEPENDENCY_MISSING: failed to launch copied binary {}: {error}",
            binary.display()
        )
    })?;
    let stdout = child.stdout.take().ok_or_else(|| {
        "NATIVE_GATE_PACKAGE_INVALID: failed to capture package smoke stdout".to_owned()
    })?;
    let stderr = child.stderr.take().ok_or_else(|| {
        "NATIVE_GATE_PACKAGE_INVALID: failed to capture package smoke stderr".to_owned()
    })?;
    let stdout_reader = spawn_bounded_reader(stdout, MAX_SMOKE_STDOUT_BYTES);
    let stderr_reader = spawn_bounded_reader(stderr, MAX_SMOKE_STDERR_BYTES);
    let started = Instant::now();

    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() >= timeout => {
                let kill_result = child.kill();
                let wait_result = child.wait();
                let (stdout, stdout_truncated) = join_bounded_reader(stdout_reader, "stdout")?;
                let (stderr, stderr_truncated) = join_bounded_reader(stderr_reader, "stderr")?;
                let kill_detail = kill_result
                    .err()
                    .map_or_else(String::new, |error| format!("; kill failed: {error}"));
                let wait_detail = wait_result
                    .err()
                    .map_or_else(String::new, |error| format!("; wait failed: {error}"));
                return Err(format!(
                    "NATIVE_GATE_PACKAGE_SMOKE_TIMEOUT: {} exceeded {} seconds{}{}; stdout: {}; stderr: {}{}{}",
                    binary.display(),
                    timeout.as_secs_f64(),
                    kill_detail,
                    wait_detail,
                    bounded_text(&stdout),
                    bounded_text(&stderr),
                    if stdout_truncated {
                        " [stdout truncated]"
                    } else {
                        ""
                    },
                    if stderr_truncated {
                        " [stderr truncated]"
                    } else {
                        ""
                    },
                ));
            }
            Ok(None) => thread::sleep(PACKAGE_SMOKE_POLL_INTERVAL.min(timeout)),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = join_bounded_reader(stdout_reader, "stdout");
                let _ = join_bounded_reader(stderr_reader, "stderr");
                return package_error(format!(
                    "failed while waiting for package smoke {}: {error}",
                    binary.display()
                ));
            }
        }
    };
    let (stdout, stdout_truncated) = join_bounded_reader(stdout_reader, "stdout")?;
    let (stderr, stderr_truncated) = join_bounded_reader(stderr_reader, "stderr")?;
    Ok(BoundedCommandOutput {
        status,
        stdout,
        stderr,
        stdout_truncated,
        stderr_truncated,
    })
}

fn spawn_bounded_reader<R>(
    mut reader: R,
    limit: usize,
) -> thread::JoinHandle<std::io::Result<(Vec<u8>, bool)>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut bytes = Vec::with_capacity(limit.min(64 * 1024));
        let mut buffer = [0_u8; 8192];
        let mut truncated = false;
        loop {
            let count = reader.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            let remaining = limit.saturating_sub(bytes.len());
            let retained = remaining.min(count);
            bytes.extend_from_slice(&buffer[..retained]);
            truncated |= retained != count;
        }
        Ok((bytes, truncated))
    })
}

fn join_bounded_reader(
    reader: thread::JoinHandle<std::io::Result<(Vec<u8>, bool)>>,
    stream: &str,
) -> Result<(Vec<u8>, bool), String> {
    reader
        .join()
        .map_err(|_| format!("NATIVE_GATE_PACKAGE_INVALID: {stream} reader panicked"))?
        .map_err(|error| {
            format!("NATIVE_GATE_PACKAGE_INVALID: failed to read package smoke {stream}: {error}")
        })
}
