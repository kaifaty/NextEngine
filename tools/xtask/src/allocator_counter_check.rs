use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use xtask::allocator_counter_kernel::{
    ALLOCATOR_COUNTER_KERNEL_METHODOLOGY_VERSION, ALLOCATOR_COUNTER_KERNEL_PREIMAGE,
    ALLOCATOR_COUNTER_KERNEL_SCHEMA_VERSION, AllocatorCounterKernelModeV1,
    AllocatorCounterKernelReportV1, AllocatorCounterKernelRootsV1,
    MAX_ALLOCATOR_COUNTER_KERNEL_JSON_BYTES,
};

const CHECK_SCHEMA_VERSION: u32 = 1;
const CHECK_METHODOLOGY_VERSION: &str = "nextengine-allocator-counter-check-v1";
const WARMUP_ROUNDS: u32 = 2;
const RETAINED_ROUNDS: u32 = 15;
const MAX_OVERHEAD_BASIS_POINTS: i64 = 300;
const MAX_RESERVED_BYTES: u64 = 64 * 1024 * 1024;
const MAX_HELPER_RUNTIME: Duration = Duration::from_secs(60);
const MAX_HELPER_STDERR_BYTES: usize = 64 * 1024;
const MAX_EVIDENCE_JSON_BYTES: usize = 1024 * 1024;
const REPORT_FILE_NAME: &str = "allocator-counter-check-v1.json";
const TEMPORARY_REPORT_FILE_NAME: &str = ".allocator-counter-check-v1.json.tmp";

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct AllocatorCounterCheckArguments {
    output: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum AllocatorCounterCheckStatusV1 {
    Pass,
    Fail,
    NotRun,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AllocatorCounterCheckRoundV1 {
    round: u32,
    warmup: bool,
    order: [AllocatorCounterKernelModeV1; 3],
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AllocatorCounterModeEvidenceV1 {
    mode: AllocatorCounterKernelModeV1,
    elapsed_nanoseconds: Vec<u64>,
    median_nanoseconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AllocatorCounterCheckReportV1 {
    schema_version: u32,
    methodology_version: String,
    status: AllocatorCounterCheckStatusV1,
    commit: String,
    worktree_clean: bool,
    toolchain_hex: String,
    target_triple: String,
    build_profile: String,
    kernel_hash: String,
    warmup_rounds: u32,
    retained_rounds: u32,
    schedule: Vec<AllocatorCounterCheckRoundV1>,
    system: Option<AllocatorCounterModeEvidenceV1>,
    instrumented_inactive: Option<AllocatorCounterModeEvidenceV1>,
    instrumented_enabled: Option<AllocatorCounterModeEvidenceV1>,
    inactive_overhead_basis_points: Option<i64>,
    enabled_overhead_basis_points: Option<i64>,
    maximum_overhead_basis_points: i64,
    reserved_bytes: Option<u64>,
    maximum_reserved_bytes: u64,
    roots: Option<AllocatorCounterKernelRootsV1>,
    diagnostics: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct HelperIdentity {
    commit: String,
    worktree_clean: bool,
    toolchain_hex: String,
    target_triple: String,
    build_profile: String,
    kernel_hash: String,
}

impl From<&AllocatorCounterKernelReportV1> for HelperIdentity {
    fn from(report: &AllocatorCounterKernelReportV1) -> Self {
        Self {
            commit: report.commit.clone(),
            worktree_clean: report.worktree_clean,
            toolchain_hex: report.toolchain_hex.clone(),
            target_triple: report.target_triple.clone(),
            build_profile: report.build_profile.clone(),
            kernel_hash: report.kernel_hash.clone(),
        }
    }
}

#[derive(Default)]
struct RetainedReports {
    system: Vec<AllocatorCounterKernelReportV1>,
    inactive: Vec<AllocatorCounterKernelReportV1>,
    enabled: Vec<AllocatorCounterKernelReportV1>,
}

impl RetainedReports {
    fn push(&mut self, report: AllocatorCounterKernelReportV1) {
        match report.mode {
            AllocatorCounterKernelModeV1::System => self.system.push(report),
            AllocatorCounterKernelModeV1::InstrumentedInactive => self.inactive.push(report),
            AllocatorCounterKernelModeV1::InstrumentedEnabled => self.enabled.push(report),
        }
    }
}

#[derive(Debug)]
struct CheckFailure {
    status: AllocatorCounterCheckStatusV1,
    diagnostic: String,
}

impl CheckFailure {
    fn not_run(code: &str, detail: impl AsRef<str>) -> Self {
        Self {
            status: AllocatorCounterCheckStatusV1::NotRun,
            diagnostic: stable_diagnostic(code, detail.as_ref()),
        }
    }

    fn fail(code: &str, detail: impl AsRef<str>) -> Self {
        Self {
            status: AllocatorCounterCheckStatusV1::Fail,
            diagnostic: stable_diagnostic(code, detail.as_ref()),
        }
    }
}

pub(crate) fn parse_arguments(
    mut arguments: impl Iterator<Item = String>,
) -> Result<AllocatorCounterCheckArguments, String> {
    let mut request = AllocatorCounterCheckArguments::default();
    while let Some(flag) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--output" if request.output.is_none() => request.output = Some(PathBuf::from(value)),
            "--output" => return Err("duplicate argument: --output".to_owned()),
            _ => return Err(format!("unexpected argument: {flag}")),
        }
    }
    Ok(request)
}

pub(crate) fn allocator_counter_check(
    root: &Path,
    request: &AllocatorCounterCheckArguments,
) -> Result<(), String> {
    let output = request
        .output
        .clone()
        .unwrap_or_else(|| root.join("target").join("allocator-counter-check"));
    let report = match execute_check(root) {
        Ok(report) => report,
        Err(failure) => failure_report(failure),
    };
    let json = serde_json::to_vec(&report)
        .map_err(|error| format!("ALLOCATOR_COUNTER_CHECK_JSON_FAILED: {error}"))?;
    if json.len() > MAX_EVIDENCE_JSON_BYTES {
        return Err("ALLOCATOR_COUNTER_CHECK_JSON_TOO_LARGE".to_owned());
    }
    publish_report(&output, &json)?;
    println!(
        "{}",
        String::from_utf8(json).map_err(|error| error.to_string())?
    );
    match report.status {
        AllocatorCounterCheckStatusV1::Pass => Ok(()),
        AllocatorCounterCheckStatusV1::Fail => {
            Err("ALLOCATOR_COUNTER_CHECK_FAILED: evidence verdict is FAIL".to_owned())
        }
        AllocatorCounterCheckStatusV1::NotRun => {
            Err("ALLOCATOR_COUNTER_CHECK_NOT_RUN: evidence is unavailable".to_owned())
        }
    }
}

fn execute_check(root: &Path) -> Result<AllocatorCounterCheckReportV1, CheckFailure> {
    xtask::allocator_counter_codegen::validate_pinned_windows_release(root)
        .map_err(|error| CheckFailure::not_run(error.code(), error.detail()))?;
    build_release_helpers(root)?;
    let helpers = helper_paths(root)?;
    let scratch = CheckScratchRoot::create(root)?;
    let expected_kernel_hash = xtask::performance::sha256_hex(ALLOCATOR_COUNTER_KERNEL_PREIMAGE);
    let mut expected_identity = None;
    let mut expected_roots = None;
    let mut retained = RetainedReports::default();
    let total_rounds = WARMUP_ROUNDS
        .checked_add(RETAINED_ROUNDS)
        .ok_or_else(|| CheckFailure::not_run("ALLOCATOR_COUNTER_CHECK_ROUND_OVERFLOW", ""))?;
    let mut schedule = Vec::with_capacity(usize::try_from(total_rounds).unwrap_or(0));
    for round in 0..total_rounds {
        let order = rotated_mode_order(round);
        schedule.push(AllocatorCounterCheckRoundV1 {
            round,
            warmup: round < WARMUP_ROUNDS,
            order,
        });
        for mode in order {
            let report = run_helper(root, &helpers, scratch.path(), mode)?;
            validate_helper_report(
                &report,
                mode,
                &expected_kernel_hash,
                &mut expected_identity,
                &mut expected_roots,
            )?;
            if round >= WARMUP_ROUNDS {
                retained.push(report);
            }
        }
    }
    scratch.finish()?;
    evaluate_retained(retained, expected_identity, expected_roots, schedule)
}

fn build_release_helpers(root: &Path) -> Result<(), CheckFailure> {
    let status = Command::new("cargo")
        .args([
            "build",
            "--locked",
            "--release",
            "-p",
            "xtask",
            "--bin",
            "allocator_counter_system",
            "--bin",
            "allocator_counter_instrumented",
        ])
        .current_dir(root)
        .status()
        .map_err(|error| {
            CheckFailure::not_run(
                "ALLOCATOR_COUNTER_CHECK_HELPER_BUILD_FAILED",
                error.to_string(),
            )
        })?;
    if !status.success() {
        return Err(CheckFailure::not_run(
            "ALLOCATOR_COUNTER_CHECK_HELPER_BUILD_FAILED",
            format!("cargo exited with {status}"),
        ));
    }
    Ok(())
}

struct HelperPaths {
    system: PathBuf,
    instrumented: PathBuf,
}

fn helper_paths(root: &Path) -> Result<HelperPaths, CheckFailure> {
    let target = env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .map_or_else(|| root.join("target"), |path| resolve_from_root(root, path));
    let suffix = env::consts::EXE_SUFFIX;
    let paths = HelperPaths {
        system: target
            .join("release")
            .join(format!("allocator_counter_system{suffix}")),
        instrumented: target
            .join("release")
            .join(format!("allocator_counter_instrumented{suffix}")),
    };
    for path in [&paths.system, &paths.instrumented] {
        let metadata = fs::symlink_metadata(path).map_err(|error| {
            CheckFailure::not_run(
                "ALLOCATOR_COUNTER_CHECK_HELPER_MISSING",
                format!("{}: {error}", path.display()),
            )
        })?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err(CheckFailure::not_run(
                "ALLOCATOR_COUNTER_CHECK_HELPER_INVALID",
                path.display().to_string(),
            ));
        }
    }
    Ok(paths)
}

fn run_helper(
    root: &Path,
    helpers: &HelperPaths,
    scratch: &Path,
    mode: AllocatorCounterKernelModeV1,
) -> Result<AllocatorCounterKernelReportV1, CheckFailure> {
    let (program, mode_argument) = match mode {
        AllocatorCounterKernelModeV1::System => (&helpers.system, None),
        AllocatorCounterKernelModeV1::InstrumentedInactive => {
            (&helpers.instrumented, Some("inactive"))
        }
        AllocatorCounterKernelModeV1::InstrumentedEnabled => {
            (&helpers.instrumented, Some("enabled"))
        }
    };
    let mut command = Command::new(program);
    command
        .current_dir(root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(mode) = mode_argument {
        command.args(["--mode", mode]);
    }
    command.arg("--state-root").arg(scratch);
    let mut child = command.spawn().map_err(|error| {
        CheckFailure::not_run(
            "ALLOCATOR_COUNTER_CHECK_HELPER_SPAWN_FAILED",
            error.to_string(),
        )
    })?;
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if started.elapsed() < MAX_HELPER_RUNTIME => {
                thread::sleep(Duration::from_millis(10));
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(CheckFailure::not_run(
                    "ALLOCATOR_COUNTER_CHECK_HELPER_TIMEOUT",
                    mode.as_str(),
                ));
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(CheckFailure::not_run(
                    "ALLOCATOR_COUNTER_CHECK_HELPER_WAIT_FAILED",
                    error.to_string(),
                ));
            }
        }
    }
    let output = child.wait_with_output().map_err(|error| {
        CheckFailure::not_run(
            "ALLOCATOR_COUNTER_CHECK_HELPER_WAIT_FAILED",
            error.to_string(),
        )
    })?;
    if output.stdout.len() > MAX_ALLOCATOR_COUNTER_KERNEL_JSON_BYTES
        || output.stderr.len() > MAX_HELPER_STDERR_BYTES
    {
        return Err(CheckFailure::not_run(
            "ALLOCATOR_COUNTER_CHECK_HELPER_OUTPUT_TOO_LARGE",
            mode.as_str(),
        ));
    }
    if !output.status.success() {
        return Err(CheckFailure::not_run(
            "ALLOCATOR_COUNTER_CHECK_HELPER_FAILED",
            String::from_utf8_lossy(&output.stderr),
        ));
    }
    serde_json::from_slice(&output.stdout).map_err(|error| {
        CheckFailure::not_run(
            "ALLOCATOR_COUNTER_CHECK_HELPER_JSON_INVALID",
            error.to_string(),
        )
    })
}

fn validate_helper_report(
    report: &AllocatorCounterKernelReportV1,
    expected_mode: AllocatorCounterKernelModeV1,
    expected_kernel_hash: &str,
    expected_identity: &mut Option<HelperIdentity>,
    expected_roots: &mut Option<AllocatorCounterKernelRootsV1>,
) -> Result<(), CheckFailure> {
    if report.schema_version != ALLOCATOR_COUNTER_KERNEL_SCHEMA_VERSION
        || report.methodology_version != ALLOCATOR_COUNTER_KERNEL_METHODOLOGY_VERSION
    {
        return Err(CheckFailure::not_run(
            "ALLOCATOR_COUNTER_CHECK_HELPER_SCHEMA_MISMATCH",
            report.mode.as_str(),
        ));
    }
    if report.mode != expected_mode {
        return Err(CheckFailure::not_run(
            "ALLOCATOR_COUNTER_CHECK_HELPER_MODE_MISMATCH",
            report.mode.as_str(),
        ));
    }
    if report.kernel_hash != expected_kernel_hash {
        return Err(CheckFailure::not_run(
            "ALLOCATOR_COUNTER_CHECK_KERNEL_MISMATCH",
            report.kernel_hash.as_str(),
        ));
    }
    if report.build_profile != "release" {
        return Err(CheckFailure::not_run(
            "ALLOCATOR_COUNTER_CHECK_RELEASE_REQUIRED",
            report.build_profile.as_str(),
        ));
    }
    if report.target_triple == "unsupported-target" {
        return Err(CheckFailure::not_run(
            "ALLOCATOR_COUNTER_CHECK_TARGET_UNSUPPORTED",
            report.target_triple.as_str(),
        ));
    }
    if report.commit == "UNKNOWN"
        || report.toolchain_hex.is_empty()
        || report.elapsed_nanoseconds == 0
    {
        return Err(CheckFailure::not_run(
            "ALLOCATOR_COUNTER_CHECK_HELPER_IDENTITY_INVALID",
            report.mode.as_str(),
        ));
    }
    validate_roots(&report.roots)?;
    validate_snapshot(report)?;
    let identity = HelperIdentity::from(report);
    match expected_identity {
        Some(expected) if expected != &identity => {
            return Err(CheckFailure::not_run(
                "ALLOCATOR_COUNTER_CHECK_IDENTITY_MISMATCH",
                report.mode.as_str(),
            ));
        }
        None => *expected_identity = Some(identity),
        _ => {}
    }
    match expected_roots {
        Some(expected) if expected != &report.roots => {
            return Err(CheckFailure::fail(
                "ALLOCATOR_COUNTER_CHECK_ROOT_MISMATCH",
                report.mode.as_str(),
            ));
        }
        None => *expected_roots = Some(report.roots.clone()),
        _ => {}
    }
    Ok(())
}

fn validate_roots(roots: &AllocatorCounterKernelRootsV1) -> Result<(), CheckFailure> {
    if roots.ticks != 900 || roots.command_body_count != 900 {
        return Err(CheckFailure::fail(
            "ALLOCATOR_COUNTER_CHECK_ROOT_MISMATCH",
            format!(
                "ticks={}, command_body_count={}",
                roots.ticks, roots.command_body_count
            ),
        ));
    }
    if roots.application_final_state_root.is_some()
        || !is_sha256(&roots.final_state_root)
        || !is_sha256(&roots.final_command_archive_root)
        || !is_sha256(&roots.final_command_identity_index_root)
    {
        return Err(CheckFailure::not_run(
            "ALLOCATOR_COUNTER_CHECK_ROOT_INVALID",
            "prepared smoke roots are malformed",
        ));
    }
    Ok(())
}

fn validate_snapshot(report: &AllocatorCounterKernelReportV1) -> Result<(), CheckFailure> {
    let expected_reserved = u64::try_from(next_process_allocation_counter::reserved_bytes())
        .map_err(|error| {
            CheckFailure::not_run(
                "ALLOCATOR_COUNTER_CHECK_RESERVED_BYTES_INVALID",
                error.to_string(),
            )
        })?;
    match (report.mode, report.allocator_snapshot) {
        (AllocatorCounterKernelModeV1::System, None) if report.reserved_bytes == 0 => Ok(()),
        (AllocatorCounterKernelModeV1::InstrumentedInactive, None)
            if report.reserved_bytes == expected_reserved =>
        {
            Ok(())
        }
        (AllocatorCounterKernelModeV1::InstrumentedEnabled, Some(snapshot))
            if report.reserved_bytes == expected_reserved =>
        {
            let count = snapshot
                .alloc_count
                .checked_add(snapshot.alloc_zeroed_count)
                .and_then(|value| value.checked_add(snapshot.realloc_count));
            let bytes = snapshot
                .alloc_bytes
                .checked_add(snapshot.alloc_zeroed_bytes)
                .and_then(|value| value.checked_add(snapshot.realloc_bytes));
            if snapshot.producer_pid == 0
                || snapshot.window_id == 0
                || snapshot.allocator_allocation_count == 0
                || snapshot.allocator_allocated_bytes == 0
                || count != Some(snapshot.allocator_allocation_count)
                || bytes != Some(snapshot.allocator_allocated_bytes)
            {
                return Err(CheckFailure::not_run(
                    "ALLOCATOR_COUNTER_CHECK_SNAPSHOT_INVALID",
                    report.mode.as_str(),
                ));
            }
            Ok(())
        }
        _ => Err(CheckFailure::not_run(
            "ALLOCATOR_COUNTER_CHECK_SNAPSHOT_INVALID",
            report.mode.as_str(),
        )),
    }
}

fn evaluate_retained(
    reports: RetainedReports,
    identity: Option<HelperIdentity>,
    roots: Option<AllocatorCounterKernelRootsV1>,
    schedule: Vec<AllocatorCounterCheckRoundV1>,
) -> Result<AllocatorCounterCheckReportV1, CheckFailure> {
    let identity = identity.ok_or_else(|| {
        CheckFailure::not_run("ALLOCATOR_COUNTER_CHECK_NO_SAMPLES", "missing identity")
    })?;
    let roots = roots.ok_or_else(|| {
        CheckFailure::not_run("ALLOCATOR_COUNTER_CHECK_NO_SAMPLES", "missing roots")
    })?;
    let expected_count = usize::try_from(RETAINED_ROUNDS).unwrap_or(0);
    if reports.system.len() != expected_count
        || reports.inactive.len() != expected_count
        || reports.enabled.len() != expected_count
    {
        return Err(CheckFailure::not_run(
            "ALLOCATOR_COUNTER_CHECK_SAMPLE_COUNT_INVALID",
            format!(
                "system={}, inactive={}, enabled={}",
                reports.system.len(),
                reports.inactive.len(),
                reports.enabled.len()
            ),
        ));
    }
    let system = summarize_mode(AllocatorCounterKernelModeV1::System, &reports.system)?;
    let inactive = summarize_mode(
        AllocatorCounterKernelModeV1::InstrumentedInactive,
        &reports.inactive,
    )?;
    let enabled = summarize_mode(
        AllocatorCounterKernelModeV1::InstrumentedEnabled,
        &reports.enabled,
    )?;
    let inactive_overhead =
        overhead_basis_points(system.median_nanoseconds, inactive.median_nanoseconds)?;
    let enabled_overhead =
        overhead_basis_points(system.median_nanoseconds, enabled.median_nanoseconds)?;
    let reserved_bytes = reports
        .enabled
        .first()
        .map(|report| report.reserved_bytes)
        .ok_or_else(|| CheckFailure::not_run("ALLOCATOR_COUNTER_CHECK_NO_SAMPLES", "enabled"))?;
    let mut diagnostics = Vec::new();
    if inactive_overhead > MAX_OVERHEAD_BASIS_POINTS {
        diagnostics.push(stable_diagnostic(
            "ALLOCATOR_COUNTER_CHECK_INACTIVE_OVERHEAD_EXCEEDED",
            &inactive_overhead.to_string(),
        ));
    }
    if enabled_overhead > MAX_OVERHEAD_BASIS_POINTS {
        diagnostics.push(stable_diagnostic(
            "ALLOCATOR_COUNTER_CHECK_ENABLED_OVERHEAD_EXCEEDED",
            &enabled_overhead.to_string(),
        ));
    }
    if reserved_bytes > MAX_RESERVED_BYTES {
        diagnostics.push(stable_diagnostic(
            "ALLOCATOR_COUNTER_CHECK_RESERVED_BYTES_EXCEEDED",
            &reserved_bytes.to_string(),
        ));
    }
    let status = if diagnostics.is_empty() {
        AllocatorCounterCheckStatusV1::Pass
    } else {
        AllocatorCounterCheckStatusV1::Fail
    };
    Ok(AllocatorCounterCheckReportV1 {
        schema_version: CHECK_SCHEMA_VERSION,
        methodology_version: CHECK_METHODOLOGY_VERSION.to_owned(),
        status,
        commit: identity.commit,
        worktree_clean: identity.worktree_clean,
        toolchain_hex: identity.toolchain_hex,
        target_triple: identity.target_triple,
        build_profile: identity.build_profile,
        kernel_hash: identity.kernel_hash,
        warmup_rounds: WARMUP_ROUNDS,
        retained_rounds: RETAINED_ROUNDS,
        schedule,
        system: Some(system),
        instrumented_inactive: Some(inactive),
        instrumented_enabled: Some(enabled),
        inactive_overhead_basis_points: Some(inactive_overhead),
        enabled_overhead_basis_points: Some(enabled_overhead),
        maximum_overhead_basis_points: MAX_OVERHEAD_BASIS_POINTS,
        reserved_bytes: Some(reserved_bytes),
        maximum_reserved_bytes: MAX_RESERVED_BYTES,
        roots: Some(roots),
        diagnostics,
    })
}

fn summarize_mode(
    mode: AllocatorCounterKernelModeV1,
    reports: &[AllocatorCounterKernelReportV1],
) -> Result<AllocatorCounterModeEvidenceV1, CheckFailure> {
    let samples = reports
        .iter()
        .map(|report| report.elapsed_nanoseconds)
        .collect::<Vec<_>>();
    let median = xtask::performance::nearest_rank_percentile(&samples, 50)
        .map_err(|error| CheckFailure::not_run("ALLOCATOR_COUNTER_CHECK_MEDIAN_FAILED", error))?;
    Ok(AllocatorCounterModeEvidenceV1 {
        mode,
        elapsed_nanoseconds: samples,
        median_nanoseconds: median,
    })
}

fn overhead_basis_points(system: u64, candidate: u64) -> Result<i64, CheckFailure> {
    if system == 0 {
        return Err(CheckFailure::not_run(
            "ALLOCATOR_COUNTER_CHECK_ZERO_SYSTEM_MEDIAN",
            "",
        ));
    }
    let delta = i128::from(candidate) - i128::from(system);
    let basis_points = delta.checked_mul(10_000).ok_or_else(|| {
        CheckFailure::not_run("ALLOCATOR_COUNTER_CHECK_OVERHEAD_OVERFLOW", "multiply")
    })? / i128::from(system);
    i64::try_from(basis_points).map_err(|error| {
        CheckFailure::not_run(
            "ALLOCATOR_COUNTER_CHECK_OVERHEAD_OVERFLOW",
            error.to_string(),
        )
    })
}

fn rotated_mode_order(round: u32) -> [AllocatorCounterKernelModeV1; 3] {
    const MODES: [AllocatorCounterKernelModeV1; 3] = [
        AllocatorCounterKernelModeV1::System,
        AllocatorCounterKernelModeV1::InstrumentedInactive,
        AllocatorCounterKernelModeV1::InstrumentedEnabled,
    ];
    let offset = usize::try_from(round % 3).unwrap_or(0);
    [
        MODES[offset],
        MODES[(offset + 1) % MODES.len()],
        MODES[(offset + 2) % MODES.len()],
    ]
}

fn failure_report(failure: CheckFailure) -> AllocatorCounterCheckReportV1 {
    AllocatorCounterCheckReportV1 {
        schema_version: CHECK_SCHEMA_VERSION,
        methodology_version: CHECK_METHODOLOGY_VERSION.to_owned(),
        status: failure.status,
        commit: "UNKNOWN".to_owned(),
        worktree_clean: false,
        toolchain_hex: String::new(),
        target_triple: "UNKNOWN".to_owned(),
        build_profile: "UNKNOWN".to_owned(),
        kernel_hash: xtask::performance::sha256_hex(ALLOCATOR_COUNTER_KERNEL_PREIMAGE),
        warmup_rounds: WARMUP_ROUNDS,
        retained_rounds: RETAINED_ROUNDS,
        schedule: Vec::new(),
        system: None,
        instrumented_inactive: None,
        instrumented_enabled: None,
        inactive_overhead_basis_points: None,
        enabled_overhead_basis_points: None,
        maximum_overhead_basis_points: MAX_OVERHEAD_BASIS_POINTS,
        reserved_bytes: None,
        maximum_reserved_bytes: MAX_RESERVED_BYTES,
        roots: None,
        diagnostics: vec![failure.diagnostic],
    }
}

struct CheckScratchRoot {
    path: Option<PathBuf>,
}

impl CheckScratchRoot {
    fn create(root: &Path) -> Result<Self, CheckFailure> {
        let base = root.join("target");
        fs::create_dir_all(&base).map_err(|error| {
            CheckFailure::not_run("ALLOCATOR_COUNTER_CHECK_SCRATCH_FAILED", error.to_string())
        })?;
        for sequence in 0..1_024_u32 {
            let path = base.join(format!(
                "allocator-counter-check-scratch-{}-{sequence}",
                std::process::id()
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self { path: Some(path) }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => {
                    return Err(CheckFailure::not_run(
                        "ALLOCATOR_COUNTER_CHECK_SCRATCH_FAILED",
                        error.to_string(),
                    ));
                }
            }
        }
        Err(CheckFailure::not_run(
            "ALLOCATOR_COUNTER_CHECK_SCRATCH_FAILED",
            "unique scratch root unavailable",
        ))
    }

    fn path(&self) -> &Path {
        self.path
            .as_deref()
            .expect("scratch root exists until finish")
    }

    fn finish(mut self) -> Result<(), CheckFailure> {
        let path = self.path.take().expect("scratch root exists until finish");
        fs::remove_dir(&path).map_err(|error| {
            CheckFailure::not_run(
                "ALLOCATOR_COUNTER_CHECK_SCRATCH_CLEANUP_FAILED",
                error.to_string(),
            )
        })
    }
}

impl Drop for CheckScratchRoot {
    fn drop(&mut self) {
        if let Some(path) = self.path.take() {
            let _ = fs::remove_dir(path);
        }
    }
}

fn publish_report(output: &Path, bytes: &[u8]) -> Result<(), String> {
    fs::create_dir_all(output).map_err(|error| {
        format!(
            "ALLOCATOR_COUNTER_CHECK_OUTPUT_INVALID: {}: {error}",
            output.display()
        )
    })?;
    let metadata = fs::symlink_metadata(output).map_err(|error| error.to_string())?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err("ALLOCATOR_COUNTER_CHECK_OUTPUT_INVALID".to_owned());
    }
    let final_path = output.join(REPORT_FILE_NAME);
    let temporary_path = output.join(TEMPORARY_REPORT_FILE_NAME);
    if final_path.exists() || temporary_path.exists() {
        return Err(format!(
            "ALLOCATOR_COUNTER_CHECK_OUTPUT_EXISTS: {}",
            final_path.display()
        ));
    }
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary_path)
        .map_err(|error| error.to_string())?;
    file.write_all(bytes).map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())?;
    drop(file);
    fs::rename(&temporary_path, &final_path).map_err(|error| error.to_string())
}

fn resolve_from_root(root: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        root.join(path)
    }
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn stable_diagnostic(code: &str, detail: &str) -> String {
    const MAX_DETAIL_CHARACTERS: usize = 512;
    const HEAD_CHARACTERS: usize = 128;
    const ELISION: &str = " ... ";
    let sanitized = detail
        .chars()
        .map(|character| {
            if matches!(character, '\r' | '\n' | '\t') {
                ' '
            } else {
                character
            }
        })
        .collect::<Vec<_>>();
    let detail = if sanitized.len() <= MAX_DETAIL_CHARACTERS {
        sanitized.into_iter().collect::<String>()
    } else {
        let tail_characters = MAX_DETAIL_CHARACTERS - HEAD_CHARACTERS - ELISION.len();
        let mut bounded = sanitized[..HEAD_CHARACTERS].iter().collect::<String>();
        bounded.push_str(ELISION);
        bounded.extend(
            sanitized[sanitized.len() - tail_characters..]
                .iter()
                .copied(),
        );
        bounded
    };
    if detail.is_empty() {
        code.to_owned()
    } else {
        format!("{code}: {detail}")
    }
}

#[cfg(test)]
mod tests;
