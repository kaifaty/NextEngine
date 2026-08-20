use std::env;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use super::source::REFERENCE_SOURCE_PATH;
use super::{
    PackageTargetNeutralRootsV3, PackagedToolRunV1, bounded_text, hash_file, package_error,
    trim_ascii_whitespace, validate_hash, validate_relative_package_path,
};

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
    let (mut command, state_root) =
        isolated_smoke_command(binary, smoke_session_root, package_root)?;
    command.args(arguments).arg("--state-root").arg(&state_root);
    let output = run_successful_smoke_command(command, binary, timeout)?;
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
    validate_presentation_contract(&report, expected_composition_root)?;
    Ok(report)
}

pub(super) fn run_packaged_tool(
    binary: &Path,
    smoke_session_root: &Path,
    package_root: &Path,
    expected: &PackageTargetNeutralRootsV3,
) -> Result<next_cli::CreatorRunCommandPassReportV1, String> {
    let (mut command, _) = isolated_smoke_command(binary, smoke_session_root, package_root)?;
    command.args(["project", "run", "--project", REFERENCE_SOURCE_PATH]);
    let output = run_successful_smoke_command(command, binary, PACKAGE_SMOKE_TIMEOUT)?;
    if output.stdout_truncated {
        return package_error(format!("{} smoke report exceeds 1 MiB", binary.display()));
    }
    let report: next_cli::CreatorRunCommandReportV1 =
        serde_json::from_slice(trim_ascii_whitespace(&output.stdout)).map_err(|error| {
            format!(
                "NATIVE_GATE_PACKAGE_INVALID: {} emitted invalid tools report JSON: {error}",
                binary.display()
            )
        })?;
    let next_cli::CreatorRunCommandReportV1::Pass(report) = report else {
        return package_error("packaged tools command reported failure");
    };
    validate_tools_report(&report, expected)?;
    Ok(*report)
}

pub(super) fn packaged_tool_run(
    binary_name: &str,
    binary: &Path,
    report: next_cli::CreatorRunCommandPassReportV1,
) -> Result<PackagedToolRunV1, String> {
    Ok(PackagedToolRunV1 {
        authoritative_state_root: report.details.runtime.authoritative_state_root,
        binary_path: format!("bin/{binary_name}"),
        binary_sha256: hash_file(binary)?,
        close_receipt_hash: report.details.runtime.close_receipt_hash,
        command: report.command,
        command_ledger_hash: report.details.runtime.command_ledger_hash,
        final_save_generation_hash: report.details.runtime.final_save_generation_hash,
        launch_status: report.status,
        project_composition_lock_hash: report.details.runtime.project_composition_lock_hash,
        source: report.details.source,
        source_project_path: REFERENCE_SOURCE_PATH.to_owned(),
        ticks: report.details.runtime.ticks,
    })
}

pub(super) fn validate_packaged_tool(
    run: &PackagedToolRunV1,
    expected_project_lock: &str,
) -> Result<(), String> {
    validate_relative_package_path(&run.binary_path)?;
    validate_relative_package_path(&run.source_project_path)?;
    for (name, value) in [
        ("tool binary", run.binary_sha256.as_str()),
        (
            "tool authoritative state root",
            run.authoritative_state_root.as_str(),
        ),
        ("tool close receipt", run.close_receipt_hash.as_str()),
        ("tool command ledger", run.command_ledger_hash.as_str()),
        (
            "tool final save generation",
            run.final_save_generation_hash.as_str(),
        ),
        (
            "tool project composition lock",
            run.project_composition_lock_hash.as_str(),
        ),
    ] {
        validate_hash(name, value)?;
    }
    if run.launch_status != "PASS"
        || run.command != "project.run"
        || run.source != "authoring"
        || run.source_project_path != REFERENCE_SOURCE_PATH
        || run.ticks != 1
        || run.project_composition_lock_hash != expected_project_lock
    {
        return package_error("Tools packaged run summary is invalid");
    }
    Ok(())
}

fn validate_tools_report(
    report: &next_cli::CreatorRunCommandPassReportV1,
    expected: &PackageTargetNeutralRootsV3,
) -> Result<(), String> {
    let project = &report.details.project;
    let runtime = &report.details.runtime;
    if report.schema_version != next_cli::CREATOR_RUN_REPORT_SCHEMA_VERSION
        || report.status != "PASS"
        || report.command != "project.run"
        || report.details.source != "authoring"
        || runtime.status != "PASS"
        || runtime.composition_root != "Headless"
        || runtime.ticks != 1
        || runtime.project_composition_lock_hash != expected.project_lock_sha256
        || project.project_lock_sha256 != expected.project_lock_sha256
        || project.schema_registry_sha256 != expected.schema_registry_sha256
        || project.content_manifest_sha256 != expected.content_manifest_sha256
        || project.world_partition_sha256 != expected.world_partition_sha256
        || project.mechanics_lock_sha256 != expected.mechanics_lock_sha256
    {
        return package_error("packaged tools command did not report the exact frozen project run");
    }
    for (name, value) in [
        (
            "tools authoritative state root",
            runtime.authoritative_state_root.as_str(),
        ),
        ("tools command ledger", runtime.command_ledger_hash.as_str()),
        ("tools close receipt", runtime.close_receipt_hash.as_str()),
        (
            "tools final save generation",
            runtime.final_save_generation_hash.as_str(),
        ),
    ] {
        validate_hash(name, value)?;
    }
    Ok(())
}

fn isolated_smoke_command(
    binary: &Path,
    smoke_session_root: &Path,
    package_root: &Path,
) -> Result<(Command, PathBuf), String> {
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
    command.current_dir(package_root);
    configure_smoke_environment(
        &mut command,
        &home,
        &local_app_data,
        &roaming_app_data,
        &xdg_state_home,
        &program_data,
        &temporary,
    );
    Ok((command, state_root))
}

fn run_successful_smoke_command(
    command: Command,
    binary: &Path,
    timeout: Duration,
) -> Result<BoundedCommandOutput, String> {
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
    Ok(output)
}

pub(super) fn validate_presentation_contract(
    report: &next_application::RunReportV1,
    expected_composition_root: &str,
) -> Result<(), String> {
    match expected_composition_root {
        "Game" => {
            if report.interactive_host_object_count == 0 {
                return package_error("packaged Game smoke reported zero interactive host objects");
            }
            let presentation = report.presentation.as_ref().ok_or_else(|| {
                "NATIVE_GATE_PACKAGE_INVALID: packaged Game smoke omitted its presentation snapshot"
                    .to_owned()
            })?;
            if presentation.target != "Interactive" {
                return package_error(format!(
                    "packaged Game smoke reported unexpected presentation target {}",
                    presentation.target
                ));
            }
            validate_hash(
                "packaged Game presentation snapshot hash",
                &presentation.snapshot_hash,
            )?;
            if presentation.object_count == 0 {
                return package_error(
                    "packaged Game smoke reported an empty presentation snapshot",
                );
            }
            if report.interactive_host_object_count != presentation.object_count {
                return package_error(format!(
                    "packaged Game smoke rendered {} objects but its presentation snapshot contains {}",
                    report.interactive_host_object_count, presentation.object_count
                ));
            }
        }
        "Headless" => {
            if report.interactive_host_object_count != 0 {
                return package_error("packaged Headless smoke reported interactive host objects");
            }
            if report.presentation.is_some() {
                return package_error(
                    "packaged Headless smoke unexpectedly reported a presentation snapshot",
                );
            }
        }
        other => {
            return package_error(format!(
                "package smoke presentation contract is undefined for composition root {other}"
            ));
        }
    }
    Ok(())
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
