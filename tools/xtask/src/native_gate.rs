mod bundle;
mod check_reports;
#[cfg(test)]
mod hardening_tests;
mod support;
#[cfg(test)]
mod tests;

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::Path;

use serde::{Deserialize, Serialize};

use support::*;

pub const NATIVE_GATE_SCHEMA_VERSION: u32 = 1;
pub const NATIVE_GATE_RUSTC_RELEASE: &str = "1.93.0";
pub const WINDOWS_TARGET_TRIPLE: &str = "x86_64-pc-windows-msvc";
pub const LINUX_TARGET_TRIPLE: &str = "x86_64-unknown-linux-gnu";

pub const NATIVE_GATE_REPORT_INVALID: &str = "NATIVE_GATE_REPORT_INVALID";
pub const NATIVE_GATE_TARGET_SET_INVALID: &str = "NATIVE_GATE_TARGET_SET_INVALID";
pub const NATIVE_GATE_COMMIT_MISMATCH: &str = "NATIVE_GATE_COMMIT_MISMATCH";
pub const NATIVE_GATE_ROOT_MISMATCH: &str = "NATIVE_GATE_ROOT_MISMATCH";
pub const NATIVE_GATE_PACKAGE_INVALID: &str = "NATIVE_GATE_PACKAGE_INVALID";
pub const NATIVE_GATE_PACKAGE_RUNTIME_PROFILE_INVALID: &str =
    "NATIVE_GATE_PACKAGE_RUNTIME_PROFILE_INVALID";
pub const NATIVE_GATE_PACKAGE_RUNTIME_DEPENDENCY_MISSING: &str =
    "NATIVE_GATE_PACKAGE_RUNTIME_DEPENDENCY_MISSING";
pub const NATIVE_GATE_PACKAGE_RUNTIME_ABI_UNSUPPORTED: &str =
    "NATIVE_GATE_PACKAGE_RUNTIME_ABI_UNSUPPORTED";
pub const NATIVE_GATE_PACKAGE_SMOKE_TIMEOUT: &str = "NATIVE_GATE_PACKAGE_SMOKE_TIMEOUT";

const TARGET_REPORT_FILE: &str = "target-report.json";
const MAX_TARGET_REPORT_BYTES: usize = 2 * 1024 * 1024;
const MAX_PACKAGE_MANIFEST_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NativeGateRunStatusV1 {
    Pass,
    Fail,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NativeGateCheckStatusV1 {
    Pass,
    Fail,
    NotRun,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum NativeGateCheckNameV1 {
    #[serde(rename = "host-check")]
    HostCheck,
    #[serde(rename = "play")]
    Play,
    #[serde(rename = "persistence-replay")]
    PersistenceReplay,
    #[serde(rename = "content-package")]
    ContentPackage,
    #[serde(rename = "platform")]
    Platform,
    #[serde(rename = "performance")]
    Performance,
    #[serde(rename = "v1-closure")]
    V1Closure,
    #[serde(rename = "v1-package")]
    V1Package,
}

impl NativeGateCheckNameV1 {
    pub const ORDERED: [Self; 8] = [
        Self::HostCheck,
        Self::Play,
        Self::PersistenceReplay,
        Self::ContentPackage,
        Self::Platform,
        Self::Performance,
        Self::V1Closure,
        Self::V1Package,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::HostCheck => "host-check",
            Self::Play => "play",
            Self::PersistenceReplay => "persistence-replay",
            Self::ContentPackage => "content-package",
            Self::Platform => "platform",
            Self::Performance => "performance",
            Self::V1Closure => "v1-closure",
            Self::V1Package => "v1-package",
        }
    }

    #[must_use]
    pub fn report_path(self) -> String {
        format!("checks/{}.json", self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeGateDiagnosticV1 {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeGateCheckRecordV1 {
    pub check: NativeGateCheckNameV1,
    pub status: NativeGateCheckStatusV1,
    pub report_path: Option<String>,
    pub report_sha256: Option<String>,
    pub diagnostic: Option<NativeGateDiagnosticV1>,
    pub elapsed_milliseconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "status",
    rename_all = "SCREAMING_SNAKE_CASE",
    deny_unknown_fields
)]
pub enum NativeGateTargetExecutionStatusV1 {
    Pass,
    NotRun { reason: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeGateClosureTargetSummaryV1 {
    pub target_triple: String,
    pub package_descriptor_hash: String,
    pub runtime_check: NativeGateTargetExecutionStatusV1,
    pub desktop_smoke: NativeGateTargetExecutionStatusV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeGateClosureTargetSetV1 {
    pub windows: NativeGateClosureTargetSummaryV1,
    pub linux: NativeGateClosureTargetSummaryV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeGatePackagedLaunchSummaryV1 {
    pub status: NativeGateCheckStatusV1,
    pub state_root: String,
    pub ledger_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeGatePackageSummaryV1 {
    pub status: NativeGateRunStatusV1,
    pub target_triple: String,
    pub relative_path: String,
    pub package_manifest_sha256: String,
    pub composition_lock_sha256: String,
    pub schema_registry_sha256: String,
    pub content_manifest_sha256: String,
    pub mechanics_lock_sha256: String,
    pub world_partition_sha256: String,
    pub game_binary_sha256: String,
    pub headless_binary_sha256: String,
    pub game: NativeGatePackagedLaunchSummaryV1,
    pub headless: NativeGatePackagedLaunchSummaryV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeGateComparableRootsV1 {
    pub project_composition_lock_hash: String,
    pub schema_registry_hash: String,
    pub content_manifest_hash: String,
    pub mechanics_lock_hash: String,
    pub world_partition_hash: String,
    pub luau_manifest_hash: String,
    pub wasm_manifest_hash: String,
    pub wit_v2_hash: String,
    pub wit_v3_hash: String,
    pub extension_compatibility_hash: String,
    pub play_state_root: String,
    pub play_ledger_hash: String,
    pub replay_state_root: String,
    pub replay_ledger_hash: String,
    pub platform_state_root: String,
    pub platform_ledger_hash: String,
    pub presentation_snapshot_hash: String,
    pub streaming_performance_hash: String,
    pub agent_performance_hash: String,
    pub packaged_game_state_root: String,
    pub packaged_game_ledger_hash: String,
    pub packaged_headless_state_root: String,
    pub packaged_headless_ledger_hash: String,
    pub closure_hash: String,
    pub windows_package_descriptor_hash: String,
    pub linux_package_descriptor_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeGateTargetReportV1 {
    pub schema_version: u32,
    pub status: NativeGateRunStatusV1,
    pub git_commit_sha: String,
    pub cargo_lock_sha256: String,
    pub rustc_release: String,
    pub target_triple: String,
    pub checks: Vec<NativeGateCheckRecordV1>,
    pub closure_targets: Option<NativeGateClosureTargetSetV1>,
    pub comparable_roots: Option<NativeGateComparableRootsV1>,
    pub package: Option<NativeGatePackageSummaryV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeGateComparedTargetSummaryV1 {
    pub target_triple: String,
    pub package_manifest_sha256: String,
    pub game_binary_sha256: String,
    pub headless_binary_sha256: String,
    pub total_elapsed_milliseconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeGateComparisonReportV1 {
    pub schema_version: u32,
    pub status: NativeGateRunStatusV1,
    pub native_gate_ready: bool,
    pub git_commit_sha: String,
    pub cargo_lock_sha256: String,
    pub rustc_release: String,
    pub windows: NativeGateComparedTargetSummaryV1,
    pub linux: NativeGateComparedTargetSummaryV1,
    pub comparable_roots: NativeGateComparableRootsV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeGateComparisonError {
    code: &'static str,
    detail: String,
}

impl NativeGateComparisonError {
    fn new(code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }

    #[must_use]
    pub const fn code(&self) -> &'static str {
        self.code
    }

    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl Display for NativeGateComparisonError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.code, self.detail)
    }
}

impl Error for NativeGateComparisonError {}

pub fn validate_native_gate_target_bundle(
    target_report_path: &Path,
) -> Result<NativeGateTargetReportV1, NativeGateComparisonError> {
    if target_report_path
        .file_name()
        .and_then(|name| name.to_str())
        != Some(TARGET_REPORT_FILE)
    {
        return Err(report_invalid(format!(
            "target report path must end with {TARGET_REPORT_FILE}"
        )));
    }
    let target_report_bytes = read_bounded_regular_file(
        target_report_path,
        MAX_TARGET_REPORT_BYTES,
        NATIVE_GATE_REPORT_INVALID,
        "target report",
    )?;
    check_reports::validate_target_report_json_shape(&target_report_bytes)?;
    let report: NativeGateTargetReportV1 = serde_json::from_slice(&target_report_bytes)
        .map_err(|error| report_invalid(format!("invalid target report JSON: {error}")))?;
    validate_native_gate_target_report(&report)?;

    let bundle_root = target_report_path
        .parent()
        .ok_or_else(|| report_invalid("target report has no bundle directory"))?;
    bundle::validate_bundle_tree(bundle_root, &report)?;
    check_reports::validate_check_reports(bundle_root, &report)?;

    if report.status == NativeGateRunStatusV1::Pass {
        let package_summary = report.package.as_ref().ok_or_else(|| {
            package_invalid("PASS target report does not contain a package summary")
        })?;
        let package_root = bundle_root.join(&package_summary.relative_path);
        let manifest =
            crate::package::validate_v1_package(&package_root).map_err(package_validation_error)?;
        let manifest_path = package_root.join(crate::package::PACKAGE_MANIFEST_FILE);
        let manifest_bytes = read_bounded_regular_file(
            &manifest_path,
            MAX_PACKAGE_MANIFEST_BYTES,
            NATIVE_GATE_PACKAGE_INVALID,
            "package manifest",
        )?;
        let actual_manifest_hash = sha256_hex(&manifest_bytes);
        if actual_manifest_hash != package_summary.package_manifest_sha256 {
            return Err(package_invalid(
                "package manifest bytes do not match package summary hash",
            ));
        }
        validate_package_manifest_summary(&report, package_summary, &manifest)?;
    }

    Ok(report)
}

pub fn validate_native_gate_target_report(
    report: &NativeGateTargetReportV1,
) -> Result<(), NativeGateComparisonError> {
    if report.schema_version != NATIVE_GATE_SCHEMA_VERSION {
        return Err(report_invalid(format!(
            "unsupported target report schema version: {}",
            report.schema_version
        )));
    }
    validate_lower_hex("git_commit_sha", &report.git_commit_sha, 40)?;
    validate_lower_hex("cargo_lock_sha256", &report.cargo_lock_sha256, 64)?;
    if report.rustc_release != NATIVE_GATE_RUSTC_RELEASE {
        return Err(report_invalid(format!(
            "rustc_release must be {NATIVE_GATE_RUSTC_RELEASE}"
        )));
    }
    if !is_shipping_target(&report.target_triple) {
        return Err(NativeGateComparisonError::new(
            NATIVE_GATE_TARGET_SET_INVALID,
            format!("unsupported target triple: {}", report.target_triple),
        ));
    }

    validate_check_records(report)?;

    if let Some(targets) = &report.closure_targets {
        validate_closure_targets(targets, &report.target_triple)?;
    }
    if let Some(roots) = &report.comparable_roots {
        validate_comparable_roots(roots)?;
        let targets = report
            .closure_targets
            .as_ref()
            .ok_or_else(|| report_invalid("comparable roots require closure target summaries"))?;
        if targets.windows.package_descriptor_hash != roots.windows_package_descriptor_hash {
            return Err(report_invalid(
                "Windows closure target descriptor does not match comparable roots",
            ));
        }
        if targets.linux.package_descriptor_hash != roots.linux_package_descriptor_hash {
            return Err(report_invalid(
                "Linux closure target descriptor does not match comparable roots",
            ));
        }
        validate_derived_comparable_roots(roots)?;
    }

    match report.status {
        NativeGateRunStatusV1::Pass => {
            if report.closure_targets.is_none() {
                return Err(report_invalid(
                    "PASS target report requires closure target summaries",
                ));
            }
            let roots = report
                .comparable_roots
                .as_ref()
                .ok_or_else(|| report_invalid("PASS target report requires comparable roots"))?;
            let package = report.package.as_ref().ok_or_else(|| {
                NativeGateComparisonError::new(
                    NATIVE_GATE_PACKAGE_INVALID,
                    "PASS target report requires package summary",
                )
            })?;
            validate_package(package, &report.target_triple, roots)?;
        }
        NativeGateRunStatusV1::Fail => {
            if report.closure_targets.is_some() || report.comparable_roots.is_some() {
                return Err(report_invalid(
                    "FAIL target report must not publish closure or comparable roots",
                ));
            }
            if report.package.is_some() {
                return Err(NativeGateComparisonError::new(
                    NATIVE_GATE_PACKAGE_INVALID,
                    "FAIL target report must not publish a package summary",
                ));
            }
        }
    }
    Ok(())
}

pub fn compare_native_gate_reports(
    first: &NativeGateTargetReportV1,
    second: &NativeGateTargetReportV1,
) -> Result<NativeGateComparisonReportV1, NativeGateComparisonError> {
    let (windows, linux) = order_target_reports(first, second)?;
    validate_native_gate_target_report(windows)?;
    validate_native_gate_target_report(linux)?;

    if windows.status != NativeGateRunStatusV1::Pass || linux.status != NativeGateRunStatusV1::Pass
    {
        return Err(report_invalid(
            "comparison requires Windows and Linux PASS target reports",
        ));
    }

    compare_environment_field(
        "git_commit_sha",
        &windows.git_commit_sha,
        &linux.git_commit_sha,
    )?;
    compare_environment_field(
        "cargo_lock_sha256",
        &windows.cargo_lock_sha256,
        &linux.cargo_lock_sha256,
    )?;
    compare_environment_field(
        "rustc_release",
        &windows.rustc_release,
        &linux.rustc_release,
    )?;

    let windows_roots = windows
        .comparable_roots
        .as_ref()
        .ok_or_else(|| report_invalid("Windows PASS report is missing comparable roots"))?;
    let linux_roots = linux
        .comparable_roots
        .as_ref()
        .ok_or_else(|| report_invalid("Linux PASS report is missing comparable roots"))?;
    compare_roots(windows_roots, linux_roots)?;

    Ok(NativeGateComparisonReportV1 {
        schema_version: NATIVE_GATE_SCHEMA_VERSION,
        status: NativeGateRunStatusV1::Pass,
        native_gate_ready: true,
        git_commit_sha: windows.git_commit_sha.clone(),
        cargo_lock_sha256: windows.cargo_lock_sha256.clone(),
        rustc_release: windows.rustc_release.clone(),
        windows: compared_target_summary(windows)?,
        linux: compared_target_summary(linux)?,
        comparable_roots: windows_roots.clone(),
    })
}

fn validate_check_records(
    report: &NativeGateTargetReportV1,
) -> Result<(), NativeGateComparisonError> {
    if report.checks.len() != NativeGateCheckNameV1::ORDERED.len() {
        return Err(report_invalid(format!(
            "expected {} check records, found {}",
            NativeGateCheckNameV1::ORDERED.len(),
            report.checks.len()
        )));
    }

    for (index, (record, expected)) in report
        .checks
        .iter()
        .zip(NativeGateCheckNameV1::ORDERED)
        .enumerate()
    {
        if record.check != expected {
            return Err(report_invalid(format!(
                "check record {index} must be {}",
                expected.as_str()
            )));
        }
        validate_check_record(record)?;
    }

    match report.status {
        NativeGateRunStatusV1::Pass => {
            if let Some(record) = report
                .checks
                .iter()
                .find(|record| record.status != NativeGateCheckStatusV1::Pass)
            {
                return Err(report_invalid(format!(
                    "PASS target report contains non-PASS check {}",
                    record.check.as_str()
                )));
            }
        }
        NativeGateRunStatusV1::Fail => {
            let failed_index = report
                .checks
                .iter()
                .position(|record| record.status == NativeGateCheckStatusV1::Fail)
                .ok_or_else(|| report_invalid("FAIL target report has no failed check"))?;
            if report.checks.iter().enumerate().any(|(index, record)| {
                let expected = if index < failed_index {
                    NativeGateCheckStatusV1::Pass
                } else if index == failed_index {
                    NativeGateCheckStatusV1::Fail
                } else {
                    NativeGateCheckStatusV1::NotRun
                };
                record.status != expected
            }) {
                return Err(report_invalid(
                    "FAIL check records must follow PASS* -> FAIL -> NOT_RUN*",
                ));
            }
        }
    }
    Ok(())
}

fn validate_check_record(
    record: &NativeGateCheckRecordV1,
) -> Result<(), NativeGateComparisonError> {
    match (&record.report_path, &record.report_sha256) {
        (Some(path), Some(hash)) => {
            if path != &record.check.report_path() || !is_portable_relative_path(path) {
                return Err(report_invalid(format!(
                    "invalid report path for {}: {path}",
                    record.check.as_str()
                )));
            }
            validate_lower_hex("check report_sha256", hash, 64)?;
        }
        (None, None) => {}
        _ => {
            return Err(report_invalid(format!(
                "{} report path and hash must be present together",
                record.check.as_str()
            )));
        }
    }

    match record.status {
        NativeGateCheckStatusV1::Pass => {
            if record.report_path.is_none() || record.diagnostic.is_some() {
                return Err(report_invalid(format!(
                    "PASS check {} requires a report and no diagnostic",
                    record.check.as_str()
                )));
            }
        }
        NativeGateCheckStatusV1::Fail => {
            if record.report_path.is_some() {
                return Err(report_invalid(format!(
                    "FAIL check {} must not publish a check report",
                    record.check.as_str()
                )));
            }
            let diagnostic = record.diagnostic.as_ref().ok_or_else(|| {
                report_invalid(format!(
                    "FAIL check {} requires a diagnostic",
                    record.check.as_str()
                ))
            })?;
            validate_diagnostic(diagnostic)?;
        }
        NativeGateCheckStatusV1::NotRun => {
            if record.report_path.is_some() {
                return Err(report_invalid(format!(
                    "NOT_RUN check {} must not have a report",
                    record.check.as_str()
                )));
            }
            let diagnostic = record.diagnostic.as_ref().ok_or_else(|| {
                report_invalid(format!(
                    "NOT_RUN check {} requires a diagnostic",
                    record.check.as_str()
                ))
            })?;
            validate_diagnostic(diagnostic)?;
            if diagnostic.code != "PRIOR_CHECK_FAILED" {
                return Err(report_invalid(format!(
                    "NOT_RUN check {} must use PRIOR_CHECK_FAILED",
                    record.check.as_str()
                )));
            }
        }
    }
    Ok(())
}

fn validate_diagnostic(
    diagnostic: &NativeGateDiagnosticV1,
) -> Result<(), NativeGateComparisonError> {
    if diagnostic.code.is_empty()
        || diagnostic.code.len() > 96
        || !diagnostic
            .code
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(report_invalid("diagnostic code is not a stable code"));
    }
    let message_chars = diagnostic.message.chars().count();
    if message_chars == 0 || message_chars > 512 {
        return Err(report_invalid(
            "diagnostic message must contain 1..=512 characters",
        ));
    }
    if diagnostic.code != "PRIOR_CHECK_FAILED"
        && !matches!(
            diagnostic.code.as_str(),
            "NATIVE_GATE_WORKTREE_DIRTY"
                | "NATIVE_GATE_HEAD_CHANGED"
                | "NATIVE_GATE_UNSUPPORTED_TARGET"
                | "NATIVE_GATE_DESKTOP_ADAPTER_DISABLED"
                | "NATIVE_GATE_CHECK_FAILED"
                | "NATIVE_GATE_REPORT_INVALID"
                | "NATIVE_GATE_TARGET_SET_INVALID"
                | "NATIVE_GATE_COMMIT_MISMATCH"
                | "NATIVE_GATE_ROOT_MISMATCH"
                | "NATIVE_GATE_PACKAGE_INVALID"
                | "NATIVE_GATE_PACKAGE_RUNTIME_PROFILE_INVALID"
                | "NATIVE_GATE_PACKAGE_RUNTIME_DEPENDENCY_MISSING"
                | "NATIVE_GATE_PACKAGE_RUNTIME_ABI_UNSUPPORTED"
                | "NATIVE_GATE_PACKAGE_SMOKE_TIMEOUT"
                | "NATIVE_GATE_OUTPUT_EXISTS"
        )
    {
        return Err(report_invalid("diagnostic code is not a native-gate code"));
    }
    Ok(())
}

fn validate_closure_targets(
    targets: &NativeGateClosureTargetSetV1,
    native_target: &str,
) -> Result<(), NativeGateComparisonError> {
    validate_closure_target(&targets.windows, WINDOWS_TARGET_TRIPLE)?;
    validate_closure_target(&targets.linux, LINUX_TARGET_TRIPLE)?;

    let (native, remote) = if native_target == WINDOWS_TARGET_TRIPLE {
        (&targets.windows, &targets.linux)
    } else {
        (&targets.linux, &targets.windows)
    };
    if native.runtime_check != NativeGateTargetExecutionStatusV1::Pass
        || native.desktop_smoke != NativeGateTargetExecutionStatusV1::Pass
    {
        return Err(report_invalid(format!(
            "native target {native_target} closure checks must PASS"
        )));
    }
    validate_remote_not_run(&remote.runtime_check, remote.target_triple.as_str())?;
    validate_remote_not_run(&remote.desktop_smoke, remote.target_triple.as_str())?;
    if remote.runtime_check != remote.desktop_smoke {
        return Err(report_invalid(format!(
            "remote target {} closure reasons must match",
            remote.target_triple
        )));
    }
    Ok(())
}

fn validate_closure_target(
    target: &NativeGateClosureTargetSummaryV1,
    expected_target: &str,
) -> Result<(), NativeGateComparisonError> {
    if target.target_triple != expected_target {
        return Err(NativeGateComparisonError::new(
            NATIVE_GATE_TARGET_SET_INVALID,
            format!(
                "closure target slot {expected_target} contains {}",
                target.target_triple
            ),
        ));
    }
    validate_lower_hex(
        "package_descriptor_hash",
        &target.package_descriptor_hash,
        64,
    )
}

fn validate_remote_not_run(
    status: &NativeGateTargetExecutionStatusV1,
    target: &str,
) -> Result<(), NativeGateComparisonError> {
    let NativeGateTargetExecutionStatusV1::NotRun { reason } = status else {
        return Err(report_invalid(format!(
            "remote target {target} closure check must be NOT_RUN"
        )));
    };
    if reason.is_empty()
        || reason.len() > 128
        || !reason
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(report_invalid(format!(
            "remote target {target} has an invalid NOT_RUN reason"
        )));
    }
    Ok(())
}

fn validate_comparable_roots(
    roots: &NativeGateComparableRootsV1,
) -> Result<(), NativeGateComparisonError> {
    for (field, value) in comparable_root_fields(roots) {
        validate_lower_hex(field, value, 64)?;
    }
    Ok(())
}

fn validate_derived_comparable_roots(
    roots: &NativeGateComparableRootsV1,
) -> Result<(), NativeGateComparisonError> {
    let expected_windows = target_package_descriptor_hash(WINDOWS_TARGET_TRIPLE, roots);
    if roots.windows_package_descriptor_hash != expected_windows {
        return Err(package_invalid(
            "Windows package descriptor does not match its package-bound roots",
        ));
    }
    let expected_linux = target_package_descriptor_hash(LINUX_TARGET_TRIPLE, roots);
    if roots.linux_package_descriptor_hash != expected_linux {
        return Err(package_invalid(
            "Linux package descriptor does not match its package-bound roots",
        ));
    }
    let expected_closure = closure_hash(roots);
    if roots.closure_hash != expected_closure {
        return Err(NativeGateComparisonError::new(
            NATIVE_GATE_ROOT_MISMATCH,
            "closure_hash does not match its declared roots",
        ));
    }
    Ok(())
}

fn validate_package(
    package: &NativeGatePackageSummaryV1,
    target: &str,
    roots: &NativeGateComparableRootsV1,
) -> Result<(), NativeGateComparisonError> {
    if package.status != NativeGateRunStatusV1::Pass {
        return Err(package_invalid("package summary status must be PASS"));
    }
    if package.target_triple != target {
        return Err(package_invalid(format!(
            "package target {} does not match report target {target}",
            package.target_triple
        )));
    }
    if package.relative_path != "package" || !is_portable_relative_path(&package.relative_path) {
        return Err(package_invalid(
            "package relative_path must be the portable path package",
        ));
    }
    for (field, value) in [
        (
            "package_manifest_sha256",
            package.package_manifest_sha256.as_str(),
        ),
        (
            "composition_lock_sha256",
            package.composition_lock_sha256.as_str(),
        ),
        (
            "schema_registry_sha256",
            package.schema_registry_sha256.as_str(),
        ),
        (
            "content_manifest_sha256",
            package.content_manifest_sha256.as_str(),
        ),
        (
            "mechanics_lock_sha256",
            package.mechanics_lock_sha256.as_str(),
        ),
        (
            "world_partition_sha256",
            package.world_partition_sha256.as_str(),
        ),
        ("game_binary_sha256", package.game_binary_sha256.as_str()),
        (
            "headless_binary_sha256",
            package.headless_binary_sha256.as_str(),
        ),
    ] {
        validate_package_hash(field, value)?;
    }
    validate_packaged_launch("game", &package.game)?;
    validate_packaged_launch("headless", &package.headless)?;

    compare_package_root(
        "composition_lock_sha256",
        &package.composition_lock_sha256,
        &roots.project_composition_lock_hash,
    )?;
    compare_package_root(
        "schema_registry_sha256",
        &package.schema_registry_sha256,
        &roots.schema_registry_hash,
    )?;
    compare_package_root(
        "content_manifest_sha256",
        &package.content_manifest_sha256,
        &roots.content_manifest_hash,
    )?;
    compare_package_root(
        "mechanics_lock_sha256",
        &package.mechanics_lock_sha256,
        &roots.mechanics_lock_hash,
    )?;
    compare_package_root(
        "world_partition_sha256",
        &package.world_partition_sha256,
        &roots.world_partition_hash,
    )?;
    compare_package_root(
        "game.state_root",
        &package.game.state_root,
        &roots.packaged_game_state_root,
    )?;
    compare_package_root(
        "game.ledger_hash",
        &package.game.ledger_hash,
        &roots.packaged_game_ledger_hash,
    )?;
    compare_package_root(
        "headless.state_root",
        &package.headless.state_root,
        &roots.packaged_headless_state_root,
    )?;
    compare_package_root(
        "headless.ledger_hash",
        &package.headless.ledger_hash,
        &roots.packaged_headless_ledger_hash,
    )
}
