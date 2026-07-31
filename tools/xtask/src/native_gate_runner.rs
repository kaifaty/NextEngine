use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use serde::Serialize;

use crate::native_gate_projection::{
    build_native_gate_package_result, validate_native_gate_closure,
};
use crate::native_gate_publish::{CooperativePublishLock, path_exists_without_following};
use crate::native_gate_schedule as schedule;
pub(crate) use crate::native_gate_target::native_shipping_target_for_host;
#[cfg(test)]
pub(crate) use crate::native_gate_target::wsl_markers_present;
use crate::*;
pub(crate) use schedule::NativeGateMatrixSchedule;
#[cfg(test)]
pub(crate) use schedule::native_gate_state_root;

fn file_hash(path: &Path) -> Result<next_contracts::ids::ContentHash, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    Ok(next_contracts::ids::content_hash_from_bytes(
        next_contracts::canonical::sha256(&bytes),
    ))
}

fn native_gate_identity(root: &Path) -> Result<NativeGateIdentity, String> {
    if !cfg!(feature = "desktop-sdl-ash") {
        return Err(
            "NATIVE_GATE_DESKTOP_ADAPTER_DISABLED: enable xtask feature desktop-sdl-ash".to_owned(),
        );
    }
    native_gate_repository_identity(root, true)
}

fn native_gate_comparison_identity(root: &Path) -> Result<NativeGateIdentity, String> {
    native_gate_repository_identity(root, false)
}

pub(crate) fn native_gate_repository_identity(
    root: &Path,
    require_native_shipping_target: bool,
) -> Result<NativeGateIdentity, String> {
    ensure_repository_root(root)?;
    ensure_clean_worktree(root)?;

    let git_commit = command_text(root, "git", &["rev-parse", "--verify", "HEAD"])?;
    if git_commit.len() != 40
        || !git_commit
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(format!(
            "NATIVE_GATE_REPORT_INVALID: git HEAD is not a lowercase full SHA-1 object ID: {git_commit}"
        ));
    }
    let git_object_format = command_text(root, "git", &["rev-parse", "--show-object-format"])?;
    if git_object_format != "sha1" {
        return Err(format!(
            "NATIVE_GATE_REPORT_INVALID: unsupported git object format {git_object_format}"
        ));
    }

    let rustc = command_text(root, "rustc", &["-vV"])?;
    let rustc_release = rustc
        .lines()
        .find_map(|line| line.strip_prefix("release: "))
        .ok_or_else(|| "NATIVE_GATE_REPORT_INVALID: rustc release missing".to_owned())?
        .to_owned();
    if rustc_release != "1.93.0" {
        return Err(format!(
            "NATIVE_GATE_REPORT_INVALID: rustc release must be exactly 1.93.0, got {rustc_release}"
        ));
    }
    let rustc_host = rustc
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .ok_or_else(|| "NATIVE_GATE_REPORT_INVALID: rustc host missing".to_owned())?
        .to_owned();
    let target_triple = if require_native_shipping_target {
        native_shipping_target_for_host(&rustc_host)?
    } else {
        rustc_host.clone()
    };

    let cargo_lock_sha256 = file_hash(&root.join("Cargo.lock"))?.to_hex();
    Ok(NativeGateIdentity {
        git_commit,
        git_object_format,
        cargo_lock_sha256,
        rustc_release,
        rustc_host,
        target_triple,
    })
}

fn ensure_repository_root(root: &Path) -> Result<(), String> {
    let repository = command_text(root, "git", &["rev-parse", "--show-toplevel"])?;
    let repository = fs::canonicalize(&repository)
        .map_err(|error| format!("NATIVE_GATE_REPORT_INVALID: {repository}: {error}"))?;
    let root = fs::canonicalize(root)
        .map_err(|error| format!("NATIVE_GATE_REPORT_INVALID: {}: {error}", root.display()))?;
    if repository != root {
        return Err(format!(
            "NATIVE_GATE_REPORT_INVALID: xtask must run at repository root {}, got {}",
            repository.display(),
            root.display()
        ));
    }
    Ok(())
}

pub(crate) fn ensure_clean_worktree(root: &Path) -> Result<(), String> {
    let status = command_text(
        root,
        "git",
        &["status", "--porcelain=v1", "--untracked-files=all"],
    )?;
    if status.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "NATIVE_GATE_WORKTREE_DIRTY: exact-commit gate requires a clean worktree; first entry: {}",
            status.lines().next().unwrap_or("unknown")
        ))
    }
}

fn command_text(root: &Path, program: &str, arguments: &[&str]) -> Result<String, String> {
    let output = run_output(root, program, arguments)?;
    String::from_utf8(output.stdout)
        .map(|text| text.trim().to_owned())
        .map_err(|error| format!("NATIVE_GATE_REPORT_INVALID: {program} output: {error}"))
}

fn native_gate_output_base(root: &Path, requested: &Path) -> Result<PathBuf, String> {
    if requested
        .components()
        .any(|component| component == Component::ParentDir)
    {
        return Err(
            "NATIVE_GATE_REPORT_INVALID: output may not contain parent traversal".to_owned(),
        );
    }
    let output = if requested.is_absolute() {
        requested.to_path_buf()
    } else {
        root.join(requested)
    };
    if output.starts_with(root) {
        let status = Command::new("git")
            .args(["check-ignore", "--quiet", "--"])
            .arg(&output)
            .current_dir(root)
            .status()
            .map_err(|error| {
                format!("NATIVE_GATE_REPORT_INVALID: failed to check ignored output: {error}")
            })?;
        if !status.success() {
            return Err(format!(
                "NATIVE_GATE_REPORT_INVALID: output inside repository must be ignored: {}",
                output.display()
            ));
        }
    }
    Ok(output)
}

fn verify_native_gate_identity(root: &Path, expected: &NativeGateIdentity) -> Result<(), String> {
    verify_native_gate_repository_identity(root, expected, true)
}

pub(crate) fn verify_native_gate_repository_identity(
    root: &Path,
    expected: &NativeGateIdentity,
    require_native_shipping_target: bool,
) -> Result<(), String> {
    let actual = native_gate_repository_identity(root, require_native_shipping_target)?;
    if &actual == expected {
        Ok(())
    } else {
        Err(format!(
            "NATIVE_GATE_HEAD_CHANGED: expected commit {} and lock {}, got commit {} and lock {}",
            expected.git_commit,
            expected.cargo_lock_sha256,
            actual.git_commit,
            actual.cargo_lock_sha256
        ))
    }
}

pub(crate) fn native_gate_run(root: &Path, requested_output: &Path) -> Result<(), String> {
    let identity = native_gate_identity(root)?;
    let output_base = native_gate_output_base(root, requested_output)?;
    let targets_directory = output_base.join("targets");
    let target_output = targets_directory.join(&identity.target_triple);
    let staging = targets_directory.join(format!(
        ".{}.staging-{}",
        identity.target_triple,
        std::process::id()
    ));
    fs::create_dir_all(&targets_directory).map_err(|error| {
        format!(
            "NATIVE_GATE_REPORT_INVALID: failed to create {}: {error}",
            targets_directory.display()
        )
    })?;
    let _publish_lock = CooperativePublishLock::acquire(&target_output)?;
    if path_exists_without_following(&target_output)? || path_exists_without_following(&staging)? {
        return Err(format!(
            "NATIVE_GATE_OUTPUT_EXISTS: target output or staging already exists: {}",
            target_output.display()
        ));
    }
    fs::create_dir(&staging).map_err(|error| {
        format!(
            "NATIVE_GATE_REPORT_INVALID: failed to create {}: {error}",
            staging.display()
        )
    })?;
    fs::create_dir(staging.join("checks")).map_err(|error| {
        format!("NATIVE_GATE_REPORT_INVALID: failed to create checks staging: {error}")
    })?;
    fs::create_dir(staging.join(".state")).map_err(|error| {
        format!("NATIVE_GATE_REPORT_INVALID: failed to create state staging: {error}")
    })?;

    let matrix = match run_native_gate_matrix(root, &staging, &identity) {
        Ok(matrix) => matrix,
        Err(failure) => {
            publish_native_gate_failure(&staging, &target_output, &identity, &failure.records)?;
            return Err(failure.error);
        }
    };

    if let Err(error) = verify_native_gate_identity(root, &identity) {
        let failure = convert_success_to_failure(matrix.records, error);
        publish_native_gate_failure(&staging, &target_output, &identity, &failure.records)?;
        return Err(failure.error);
    }
    remove_owned_directory(&staging.join(".state"))?;
    let report = NativeGateTargetReportV1 {
        schema_version: NATIVE_GATE_SCHEMA_VERSION,
        status: NativeGateRunStatusV1::Pass,
        git_commit_sha: identity.git_commit.clone(),
        cargo_lock_sha256: identity.cargo_lock_sha256.clone(),
        rustc_release: identity.rustc_release.clone(),
        target_triple: identity.target_triple.clone(),
        checks: matrix.records,
        closure_targets: Some(matrix.closure_targets),
        comparable_roots: Some(matrix.comparable_roots),
        package: Some(matrix.package),
    };
    publish_native_gate_target_report(&staging, &target_output, &report)?;
    println!(
        "{}",
        serde_json::to_string(&report)
            .map_err(|error| format!("NATIVE_GATE_REPORT_INVALID: {error}"))?
    );
    Ok(())
}

fn run_native_gate_matrix(
    root: &Path,
    staging: &Path,
    identity: &NativeGateIdentity,
) -> Result<NativeGateMatrixSuccess, NativeGateCheckFailure> {
    let mut schedule = NativeGateMatrixSchedule::new(staging);
    macro_rules! run_check {
        ($name:expr, $operation:expr) => {
            match schedule.run($name, |state_root, started| {
                execute_native_gate_check(
                    root, staging, identity, $name, state_root, started, $operation,
                )
            }) {
                Ok(value) => value,
                Err(failure) => return Err(failure),
            }
        };
    }

    let _host = run_check!(NativeGateCheckNameV1::HostCheck, |state_root| {
        host_check_report(root, Some(state_root))
    });
    let play = run_check!(NativeGateCheckNameV1::Play, |state_root| {
        play_report(Some(state_root))
    });
    let replay = run_check!(NativeGateCheckNameV1::PersistenceReplay, |state_root| {
        persistence_replay_report(
            next_verification::PersistenceReplayBackend::Reference,
            Some(state_root),
        )
    });
    let content = run_check!(NativeGateCheckNameV1::ContentPackage, |state_root| {
        content_package_report(Some(state_root))
    });
    let platform = run_check!(NativeGateCheckNameV1::Platform, |state_root| {
        let report = platform_report(Some(state_root))?;
        if report.details.sdl_ash_candidate != "PASS" {
            return Err(format!(
                "NATIVE_GATE_DESKTOP_ADAPTER_DISABLED: native platform candidate returned {}",
                report.details.sdl_ash_candidate
            ));
        }
        Ok(report)
    });
    let performance = run_check!(NativeGateCheckNameV1::Performance, |state_root| {
        crate::performance_command::performance_report(root, Some(state_root))
    });
    let closure = run_check!(NativeGateCheckNameV1::V1Closure, |state_root| {
        let report = v1_closure_report(Some(state_root))?;
        let targets =
            validate_native_gate_closure(identity, &report, &play, &replay, &content, &platform)?;
        Ok(NativeGateClosureCheckResult { report, targets })
    });
    let package = run_check!(NativeGateCheckNameV1::V1Package, |_state_root| {
        let build = xtask::package::build_v1_package(root, &staging.join("package"))?;
        build_native_gate_package_result(
            identity,
            &build,
            &play,
            &replay,
            &content,
            &platform,
            &performance,
            &closure,
        )
    });
    let records = schedule.finish()?;

    Ok(NativeGateMatrixSuccess {
        records,
        closure_targets: package.closure_targets,
        comparable_roots: package.comparable_roots,
        package: package.package,
    })
}

fn execute_native_gate_check<T: Serialize>(
    root: &Path,
    staging: &Path,
    identity: &NativeGateIdentity,
    check: NativeGateCheckNameV1,
    state_root: &Path,
    started: Instant,
    operation: impl FnOnce(&Path) -> Result<T, String>,
) -> Result<(T, NativeGateCheckRecordV1), Box<NativeGateCheckExecutionFailure>> {
    let outcome = run_with_identity_verification(
        || verify_native_gate_identity(root, identity),
        || operation(state_root),
    )
    .and_then(|value| {
        let bytes = serde_json::to_vec(&value)
            .map_err(|error| format!("failed to serialize {}: {error}", check.as_str()))?;
        let relative_path = check.report_path();
        let path = staging.join(&relative_path);
        fs::write(&path, &bytes).map_err(|error| {
            format!(
                "failed to write {} report {}: {error}",
                check.as_str(),
                path.display()
            )
        })?;
        Ok((value, relative_path, hash_bytes(&bytes)))
    });
    let elapsed_milliseconds = elapsed_milliseconds(started);
    match outcome {
        Ok((value, relative_path, report_sha256)) => Ok((
            value,
            NativeGateCheckRecordV1 {
                check,
                status: NativeGateCheckStatusV1::Pass,
                report_path: Some(relative_path),
                report_sha256: Some(report_sha256),
                diagnostic: None,
                elapsed_milliseconds,
            },
        )),
        Err(error) => Err(native_gate_execution_failure(check, error, started)),
    }
}

pub(crate) fn native_gate_execution_failure(
    check: NativeGateCheckNameV1,
    error: String,
    started: Instant,
) -> Box<NativeGateCheckExecutionFailure> {
    let error = native_gate_check_error(check, error);
    let record = NativeGateCheckRecordV1 {
        check,
        status: NativeGateCheckStatusV1::Fail,
        report_path: None,
        report_sha256: None,
        diagnostic: Some(NativeGateDiagnosticV1 {
            code: diagnostic_code(&error).to_owned(),
            message: bounded_diagnostic_message(&error),
        }),
        elapsed_milliseconds: elapsed_milliseconds(started),
    };
    Box::new(NativeGateCheckExecutionFailure { record, error })
}

pub(crate) fn run_with_identity_verification<T>(
    mut verify: impl FnMut() -> Result<(), String>,
    operation: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    verify()?;
    let operation_result = operation();
    let post_identity_result = verify();
    match post_identity_result {
        Ok(()) => operation_result,
        Err(identity_error) => Err(identity_error),
    }
}

fn native_gate_check_error(check: NativeGateCheckNameV1, error: String) -> String {
    if error.starts_with("NATIVE_GATE_") {
        error
    } else {
        format!("NATIVE_GATE_CHECK_FAILED: {}: {}", check.as_str(), error)
    }
}

fn elapsed_milliseconds(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
}

fn hash_bytes(bytes: &[u8]) -> String {
    next_contracts::ids::content_hash_from_bytes(next_contracts::canonical::sha256(bytes)).to_hex()
}

fn bounded_diagnostic_message(error: &str) -> String {
    error.chars().take(512).collect()
}

fn convert_success_to_failure(
    mut records: Vec<NativeGateCheckRecordV1>,
    error: String,
) -> NativeGateCheckFailure {
    if let Some(record) = records.last_mut() {
        record.status = NativeGateCheckStatusV1::Fail;
        record.report_path = None;
        record.report_sha256 = None;
        record.diagnostic = Some(NativeGateDiagnosticV1 {
            code: diagnostic_code(&error).to_owned(),
            message: bounded_diagnostic_message(&error),
        });
    }
    NativeGateCheckFailure { records, error }
}

pub(crate) fn publish_native_gate_failure(
    staging: &Path,
    target_output: &Path,
    identity: &NativeGateIdentity,
    completed_records: &[NativeGateCheckRecordV1],
) -> Result<(), String> {
    if path_exists_without_following(target_output)? {
        return Err(format!(
            "NATIVE_GATE_OUTPUT_EXISTS: target output already exists: {}",
            target_output.display()
        ));
    }
    prepare_failure_staging(staging, completed_records)?;

    let checks = complete_native_gate_failure_records(completed_records)?;
    let report = NativeGateTargetReportV1 {
        schema_version: NATIVE_GATE_SCHEMA_VERSION,
        status: NativeGateRunStatusV1::Fail,
        git_commit_sha: identity.git_commit.clone(),
        cargo_lock_sha256: identity.cargo_lock_sha256.clone(),
        rustc_release: identity.rustc_release.clone(),
        target_triple: identity.target_triple.clone(),
        checks,
        closure_targets: None,
        comparable_roots: None,
        package: None,
    };
    publish_native_gate_target_report(staging, target_output, &report)
}

fn prepare_failure_staging(
    staging: &Path,
    completed_records: &[NativeGateCheckRecordV1],
) -> Result<(), String> {
    let metadata = fs::symlink_metadata(staging).map_err(|error| {
        format!(
            "NATIVE_GATE_REPORT_INVALID: failed to inspect staging {}: {error}",
            staging.display()
        )
    })?;
    if !metadata.is_dir() || is_reparse_or_symlink(&metadata) {
        return Err(
            "NATIVE_GATE_REPORT_INVALID: failure staging must be a regular directory".to_owned(),
        );
    }

    for entry in fs::read_dir(staging).map_err(|error| {
        format!(
            "NATIVE_GATE_REPORT_INVALID: failed to enumerate staging {}: {error}",
            staging.display()
        )
    })? {
        let entry = entry.map_err(|error| {
            format!("NATIVE_GATE_REPORT_INVALID: failed to enumerate staging entry: {error}")
        })?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            return Err(
                "NATIVE_GATE_REPORT_INVALID: staging contains a non-UTF-8 entry".to_owned(),
            );
        };
        match name {
            ".state" | "package" => remove_owned_directory(&entry.path())?,
            ".package.publish.lock" => remove_owned_file(&entry.path())?,
            name if is_package_transient_directory(name) => {
                remove_owned_directory(&entry.path())?;
            }
            "checks" => {
                let metadata = fs::symlink_metadata(entry.path()).map_err(|error| {
                    format!("NATIVE_GATE_REPORT_INVALID: failed to inspect checks staging: {error}")
                })?;
                if !metadata.is_dir() || is_reparse_or_symlink(&metadata) {
                    return Err(
                        "NATIVE_GATE_REPORT_INVALID: checks staging must be a regular directory"
                            .to_owned(),
                    );
                }
            }
            _ => {
                return Err(format!(
                    "NATIVE_GATE_REPORT_INVALID: unexpected failure staging entry {name}"
                ));
            }
        }
    }

    let checks_directory = staging.join("checks");
    if !checks_directory.is_dir() {
        return Err(
            "NATIVE_GATE_REPORT_INVALID: failure staging is missing checks directory".to_owned(),
        );
    }
    for record in completed_records
        .iter()
        .filter(|record| record.status == NativeGateCheckStatusV1::Fail)
    {
        let path = staging.join(record.check.report_path());
        if path_exists_without_following(&path)? {
            let metadata = fs::symlink_metadata(&path).map_err(|error| {
                format!(
                    "NATIVE_GATE_REPORT_INVALID: failed to inspect failed check report: {error}"
                )
            })?;
            if !metadata.is_file() || is_reparse_or_symlink(&metadata) {
                return Err(format!(
                    "NATIVE_GATE_REPORT_INVALID: failed check report path is not a regular file: {}",
                    path.display()
                ));
            }
            fs::remove_file(&path).map_err(|error| {
                format!(
                    "NATIVE_GATE_REPORT_INVALID: failed to remove disqualified check report: {error}"
                )
            })?;
        }
    }
    validate_failure_check_inventory(&checks_directory, completed_records)
}

fn is_package_transient_directory(name: &str) -> bool {
    [".package.staging-", ".package.smoke-"]
        .iter()
        .any(|prefix| {
            name.strip_prefix(prefix).is_some_and(|suffix| {
                !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_digit())
            })
        })
}

fn remove_owned_file(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        format!(
            "NATIVE_GATE_REPORT_INVALID: failed to inspect transient {}: {error}",
            path.display()
        )
    })?;
    if !metadata.is_file() || is_reparse_or_symlink(&metadata) {
        return Err(format!(
            "NATIVE_GATE_REPORT_INVALID: transient is not a regular file: {}",
            path.display()
        ));
    }
    fs::remove_file(path).map_err(|error| {
        format!(
            "NATIVE_GATE_REPORT_INVALID: failed to remove transient {}: {error}",
            path.display()
        )
    })
}

fn validate_failure_check_inventory(
    checks_directory: &Path,
    completed_records: &[NativeGateCheckRecordV1],
) -> Result<(), String> {
    let expected = completed_records
        .iter()
        .filter_map(|record| record.report_path.as_deref())
        .map(|path| path.strip_prefix("checks/").unwrap_or(path).to_owned())
        .collect::<BTreeSet<_>>();
    let mut actual = BTreeSet::new();
    for entry in fs::read_dir(checks_directory).map_err(|error| {
        format!("NATIVE_GATE_REPORT_INVALID: failed to enumerate checks staging: {error}")
    })? {
        let entry = entry.map_err(|error| {
            format!("NATIVE_GATE_REPORT_INVALID: failed to enumerate checks entry: {error}")
        })?;
        let metadata = fs::symlink_metadata(entry.path()).map_err(|error| {
            format!("NATIVE_GATE_REPORT_INVALID: failed to inspect checks entry: {error}")
        })?;
        if !metadata.is_file() || is_reparse_or_symlink(&metadata) {
            return Err(format!(
                "NATIVE_GATE_REPORT_INVALID: checks staging entry is not a regular file: {}",
                entry.path().display()
            ));
        }
        let name = entry
            .file_name()
            .to_str()
            .ok_or_else(|| {
                "NATIVE_GATE_REPORT_INVALID: checks staging contains a non-UTF-8 file".to_owned()
            })?
            .to_owned();
        actual.insert(name);
    }
    if actual == expected {
        Ok(())
    } else {
        Err(
            "NATIVE_GATE_REPORT_INVALID: checks staging inventory contains unexpected files"
                .to_owned(),
        )
    }
}

pub(crate) fn complete_native_gate_failure_records(
    completed_records: &[NativeGateCheckRecordV1],
) -> Result<Vec<NativeGateCheckRecordV1>, String> {
    let mut checks = completed_records.to_vec();
    let failed = checks
        .iter()
        .find(|record| record.status == NativeGateCheckStatusV1::Fail)
        .map(|record| record.check.as_str().to_owned())
        .ok_or_else(|| {
            "NATIVE_GATE_REPORT_INVALID: controlled failure has no failed check".to_owned()
        })?;
    for check in NativeGateCheckNameV1::ORDERED.iter().skip(checks.len()) {
        checks.push(NativeGateCheckRecordV1 {
            check: *check,
            status: NativeGateCheckStatusV1::NotRun,
            report_path: None,
            report_sha256: None,
            diagnostic: Some(NativeGateDiagnosticV1 {
                code: "PRIOR_CHECK_FAILED".to_owned(),
                message: format!("not run because {failed} failed"),
            }),
            elapsed_milliseconds: 0,
        });
    }
    Ok(checks)
}

fn publish_native_gate_target_report(
    staging: &Path,
    target_output: &Path,
    report: &NativeGateTargetReportV1,
) -> Result<(), String> {
    xtask::native_gate::validate_native_gate_target_report(report)
        .map_err(|error| error.to_string())?;
    let bytes = serde_json::to_vec(report)
        .map_err(|error| format!("NATIVE_GATE_REPORT_INVALID: {error}"))?;
    let report_path = staging.join("target-report.json");
    fs::write(&report_path, bytes).map_err(|error| {
        format!(
            "NATIVE_GATE_REPORT_INVALID: failed to write {}: {error}",
            report_path.display()
        )
    })?;
    xtask::native_gate::validate_native_gate_target_bundle(&report_path)
        .map_err(|error| error.to_string())?;
    if path_exists_without_following(target_output)? {
        return Err(format!(
            "NATIVE_GATE_OUTPUT_EXISTS: target output appeared during execution: {}",
            target_output.display()
        ));
    }
    fs::rename(staging, target_output).map_err(|error| {
        format!(
            "NATIVE_GATE_REPORT_INVALID: failed to atomically publish {}: {error}",
            target_output.display()
        )
    })
}

fn remove_owned_directory(path: &Path) -> Result<(), String> {
    if !path_exists_without_following(path)? {
        return Ok(());
    }
    let mut entries = 0_usize;
    validate_owned_tree_for_removal(path, 0, &mut entries)?;
    fs::remove_dir_all(path).map_err(|error| {
        format!(
            "NATIVE_GATE_REPORT_INVALID: failed to remove transient {}: {error}",
            path.display()
        )
    })
}

fn validate_owned_tree_for_removal(
    path: &Path,
    depth: usize,
    entries: &mut usize,
) -> Result<(), String> {
    if depth > 32 || *entries > 100_000 {
        return Err(
            "NATIVE_GATE_REPORT_INVALID: transient cleanup bounds were exceeded".to_owned(),
        );
    }
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        format!(
            "NATIVE_GATE_REPORT_INVALID: failed to inspect transient {}: {error}",
            path.display()
        )
    })?;
    if is_reparse_or_symlink(&metadata) {
        return Err(format!(
            "NATIVE_GATE_REPORT_INVALID: transient must not contain symlinks or reparse points: {}",
            path.display()
        ));
    }
    if metadata.is_file() {
        *entries += 1;
        return Ok(());
    }
    if !metadata.is_dir() {
        return Err(format!(
            "NATIVE_GATE_REPORT_INVALID: transient contains a special file: {}",
            path.display()
        ));
    }
    *entries += 1;
    for entry in fs::read_dir(path).map_err(|error| {
        format!(
            "NATIVE_GATE_REPORT_INVALID: failed to enumerate transient {}: {error}",
            path.display()
        )
    })? {
        let entry = entry.map_err(|error| {
            format!("NATIVE_GATE_REPORT_INVALID: failed to enumerate transient: {error}")
        })?;
        validate_owned_tree_for_removal(&entry.path(), depth + 1, entries)?;
    }
    Ok(())
}

fn is_reparse_or_symlink(metadata: &fs::Metadata) -> bool {
    if metadata.file_type().is_symlink() {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
    }
    #[cfg(not(windows))]
    {
        false
    }
}

pub(crate) fn native_gate_compare(
    root: &Path,
    request: &NativeGateCompareArguments,
) -> Result<(), String> {
    let identity = native_gate_comparison_identity(root)?;
    let windows_path = native_gate_input_path(root, &request.windows)?;
    let linux_path = native_gate_input_path(root, &request.linux)?;
    let windows = xtask::native_gate::validate_native_gate_target_bundle(&windows_path)
        .map_err(|error| error.to_string())?;
    let linux = xtask::native_gate::validate_native_gate_target_bundle(&linux_path)
        .map_err(|error| error.to_string())?;
    validate_named_target_slots(&windows, &linux)?;
    verify_report_checkout_identity(&identity, &windows)?;
    verify_report_checkout_identity(&identity, &linux)?;
    let comparison = xtask::native_gate::compare_native_gate_reports(&windows, &linux)
        .map_err(|error| error.to_string())?;

    let output = native_gate_output_base(root, &request.output)?;
    let parent = output.parent().ok_or_else(|| {
        "NATIVE_GATE_REPORT_INVALID: comparison output has no parent directory".to_owned()
    })?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "NATIVE_GATE_REPORT_INVALID: failed to create {}: {error}",
            parent.display()
        )
    })?;
    let _publish_lock = CooperativePublishLock::acquire(&output)?;
    if path_exists_without_following(&output)? {
        return Err(format!(
            "NATIVE_GATE_OUTPUT_EXISTS: comparison output already exists: {}",
            output.display()
        ));
    }
    let output_name = output
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            "NATIVE_GATE_REPORT_INVALID: comparison output requires a UTF-8 file name".to_owned()
        })?;
    let staging = parent.join(format!(".{output_name}.staging-{}", std::process::id()));
    if path_exists_without_following(&staging)? {
        return Err(format!(
            "NATIVE_GATE_OUTPUT_EXISTS: comparison staging already exists: {}",
            staging.display()
        ));
    }
    let bytes = serde_json::to_vec(&comparison)
        .map_err(|error| format!("NATIVE_GATE_REPORT_INVALID: {error}"))?;
    fs::write(&staging, &bytes).map_err(|error| {
        format!(
            "NATIVE_GATE_REPORT_INVALID: failed to write {}: {error}",
            staging.display()
        )
    })?;
    if let Err(error) = verify_native_gate_repository_identity(root, &identity, false) {
        let _ = fs::remove_file(&staging);
        return Err(error);
    }
    if path_exists_without_following(&output)? {
        let _ = fs::remove_file(&staging);
        return Err(format!(
            "NATIVE_GATE_OUTPUT_EXISTS: comparison output appeared during execution: {}",
            output.display()
        ));
    }
    fs::rename(&staging, &output).map_err(|error| {
        format!(
            "NATIVE_GATE_REPORT_INVALID: failed to atomically publish {}: {error}",
            output.display()
        )
    })?;
    println!(
        "{}",
        serde_json::to_string(&comparison)
            .map_err(|error| format!("NATIVE_GATE_REPORT_INVALID: {error}"))?
    );
    Ok(())
}

pub(crate) fn validate_named_target_slots(
    windows: &NativeGateTargetReportV1,
    linux: &NativeGateTargetReportV1,
) -> Result<(), String> {
    if windows.target_triple == WINDOWS_TARGET_TRIPLE && linux.target_triple == LINUX_TARGET_TRIPLE
    {
        Ok(())
    } else {
        Err(format!(
            "NATIVE_GATE_TARGET_SET_INVALID: --windows must contain {WINDOWS_TARGET_TRIPLE} and --linux must contain {LINUX_TARGET_TRIPLE}"
        ))
    }
}

fn native_gate_input_path(root: &Path, requested: &Path) -> Result<PathBuf, String> {
    if requested
        .components()
        .any(|component| component == Component::ParentDir)
    {
        return Err(
            "NATIVE_GATE_REPORT_INVALID: input path may not contain parent traversal".to_owned(),
        );
    }
    Ok(if requested.is_absolute() {
        requested.to_path_buf()
    } else {
        root.join(requested)
    })
}

fn verify_report_checkout_identity(
    identity: &NativeGateIdentity,
    report: &NativeGateTargetReportV1,
) -> Result<(), String> {
    if report.git_commit_sha != identity.git_commit
        || report.cargo_lock_sha256 != identity.cargo_lock_sha256
        || report.rustc_release != identity.rustc_release
    {
        return Err(format!(
            "NATIVE_GATE_COMMIT_MISMATCH: report {} does not match clean checkout commit {}, Cargo.lock or Rust {}",
            report.target_triple, identity.git_commit, identity.rustc_release
        ));
    }
    Ok(())
}
