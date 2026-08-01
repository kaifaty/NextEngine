use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::{Deserialize, Serialize};

pub const ALLOCATOR_COUNTER_KERNEL_SCHEMA_VERSION: u32 = 1;
pub const ALLOCATOR_COUNTER_KERNEL_METHODOLOGY_VERSION: &str =
    "nextengine-allocator-counter-kernel-v1";
pub const ALLOCATOR_COUNTER_KERNEL_PREIMAGE: &[u8] =
    b"nextengine.allocator-counter-kernel.v1:prepared-live-runtime:900-driver-ticks:run-measured-only";
pub const MAX_ALLOCATOR_COUNTER_KERNEL_JSON_BYTES: usize = 64 * 1024;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AllocatorCounterKernelModeV1 {
    System,
    InstrumentedInactive,
    InstrumentedEnabled,
}

impl AllocatorCounterKernelModeV1 {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::InstrumentedInactive => "instrumented-inactive",
            Self::InstrumentedEnabled => "instrumented-enabled",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AllocatorCounterKernelRootsV1 {
    pub ticks: u64,
    pub command_body_count: u64,
    pub final_state_root: String,
    pub final_command_archive_root: String,
    pub final_command_identity_index_root: String,
    pub application_final_state_root: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AllocatorCounterSnapshotV1 {
    pub producer_pid: u32,
    pub window_id: u64,
    pub alloc_count: u64,
    pub alloc_bytes: u64,
    pub alloc_zeroed_count: u64,
    pub alloc_zeroed_bytes: u64,
    pub realloc_count: u64,
    pub realloc_bytes: u64,
    pub allocator_allocation_count: u64,
    pub allocator_allocated_bytes: u64,
}

impl From<next_process_allocation_counter::AllocationSnapshot> for AllocatorCounterSnapshotV1 {
    fn from(snapshot: next_process_allocation_counter::AllocationSnapshot) -> Self {
        Self {
            producer_pid: snapshot.producer_pid,
            window_id: snapshot.window_id,
            alloc_count: snapshot.alloc_count,
            alloc_bytes: snapshot.alloc_bytes,
            alloc_zeroed_count: snapshot.alloc_zeroed_count,
            alloc_zeroed_bytes: snapshot.alloc_zeroed_bytes,
            realloc_count: snapshot.realloc_count,
            realloc_bytes: snapshot.realloc_bytes,
            allocator_allocation_count: snapshot.allocator_allocation_count,
            allocator_allocated_bytes: snapshot.allocator_allocated_bytes,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AllocatorCounterKernelReportV1 {
    pub schema_version: u32,
    pub methodology_version: String,
    pub mode: AllocatorCounterKernelModeV1,
    pub kernel_hash: String,
    pub commit: String,
    pub worktree_clean: bool,
    pub toolchain_hex: String,
    pub target_triple: String,
    pub build_profile: String,
    pub elapsed_nanoseconds: u64,
    pub roots: AllocatorCounterKernelRootsV1,
    pub allocator_snapshot: Option<AllocatorCounterSnapshotV1>,
    pub reserved_bytes: u64,
}

pub fn run_system_helper(arguments: impl Iterator<Item = String>) -> Result<(), String> {
    let state_root = parse_system_arguments(arguments)?;
    emit_kernel_report(run_kernel(
        AllocatorCounterKernelModeV1::System,
        &state_root,
    )?)
}

pub fn run_instrumented_helper(arguments: impl Iterator<Item = String>) -> Result<(), String> {
    let (mode, state_root) = parse_instrumented_arguments(arguments)?;
    emit_kernel_report(run_kernel(mode, &state_root)?)
}

fn run_kernel(
    mode: AllocatorCounterKernelModeV1,
    state_root: &Path,
) -> Result<AllocatorCounterKernelReportV1, String> {
    let mut prepared = next_verification::prepare_live_runtime_performance_check_in(state_root)
        .map_err(|error| format!("ALLOCATOR_COUNTER_KERNEL_PREPARE_FAILED: {error}"))?;
    if prepared.ticks() != 900 {
        return Err(format!(
            "ALLOCATOR_COUNTER_KERNEL_TICK_MISMATCH: expected 900, got {}",
            prepared.ticks()
        ));
    }
    match mode {
        AllocatorCounterKernelModeV1::System
        | AllocatorCounterKernelModeV1::InstrumentedInactive => {
            let started = Instant::now();
            let measurement = prepared.run_measured();
            let elapsed_nanoseconds = elapsed_nanoseconds(started);
            let report = prepared
                .finish(measurement)
                .map_err(|error| format!("ALLOCATOR_COUNTER_KERNEL_FINISH_FAILED: {error}"))?;
            kernel_report(mode, elapsed_nanoseconds?, report, None)
        }
        AllocatorCounterKernelModeV1::InstrumentedEnabled => {
            let allocation = next_process_allocation_counter::begin().map_err(|error| {
                format!("ALLOCATOR_COUNTER_KERNEL_BEGIN_FAILED: {}", error.as_str())
            })?;
            let started = Instant::now();
            let measurement = prepared.run_measured();
            let elapsed_nanoseconds = elapsed_nanoseconds(started);
            let allocation_snapshot = allocation.finish();
            let report = prepared.finish(measurement);
            let elapsed_nanoseconds = elapsed_nanoseconds?;
            let report = report
                .map_err(|error| format!("ALLOCATOR_COUNTER_KERNEL_FINISH_FAILED: {error}"))?;
            let allocation_snapshot = allocation_snapshot.map_err(|error| {
                format!(
                    "ALLOCATOR_COUNTER_KERNEL_ALLOCATION_FINISH_FAILED: {}",
                    error.as_str()
                )
            })?;
            kernel_report(
                mode,
                elapsed_nanoseconds,
                report,
                Some(allocation_snapshot.into()),
            )
        }
    }
}

fn kernel_report(
    mode: AllocatorCounterKernelModeV1,
    elapsed_nanoseconds: u64,
    report: next_verification::LiveRuntimePerformanceReport,
    allocator_snapshot: Option<AllocatorCounterSnapshotV1>,
) -> Result<AllocatorCounterKernelReportV1, String> {
    let reserved_bytes = match mode {
        AllocatorCounterKernelModeV1::System => 0,
        AllocatorCounterKernelModeV1::InstrumentedInactive
        | AllocatorCounterKernelModeV1::InstrumentedEnabled => {
            u64::try_from(next_process_allocation_counter::reserved_bytes())
                .map_err(|error| error.to_string())?
        }
    };
    Ok(AllocatorCounterKernelReportV1 {
        schema_version: ALLOCATOR_COUNTER_KERNEL_SCHEMA_VERSION,
        methodology_version: ALLOCATOR_COUNTER_KERNEL_METHODOLOGY_VERSION.to_owned(),
        mode,
        kernel_hash: crate::performance::sha256_hex(ALLOCATOR_COUNTER_KERNEL_PREIMAGE),
        commit: env!("NEXTENGINE_BUILD_COMMIT").to_owned(),
        worktree_clean: env!("NEXTENGINE_BUILD_WORKTREE_CLEAN") == "true",
        toolchain_hex: env!("NEXTENGINE_BUILD_TOOLCHAIN_HEX").to_owned(),
        target_triple: target_triple().to_owned(),
        build_profile: env!("NEXTENGINE_BUILD_PROFILE").to_owned(),
        elapsed_nanoseconds,
        roots: AllocatorCounterKernelRootsV1 {
            ticks: report.ticks,
            command_body_count: report.command_body_count,
            final_state_root: report.final_state_root.to_hex(),
            final_command_archive_root: report.final_command_archive_root.to_hex(),
            final_command_identity_index_root: report.final_command_identity_index_root.to_hex(),
            application_final_state_root: report
                .application_final_state_root
                .map(|root| root.to_hex()),
        },
        allocator_snapshot,
        reserved_bytes,
    })
}

fn elapsed_nanoseconds(started: Instant) -> Result<u64, String> {
    u64::try_from(started.elapsed().as_nanos())
        .map_err(|error| format!("ALLOCATOR_COUNTER_KERNEL_DURATION_OVERFLOW: {error}"))
}

fn emit_kernel_report(report: AllocatorCounterKernelReportV1) -> Result<(), String> {
    let json = serde_json::to_string(&report)
        .map_err(|error| format!("ALLOCATOR_COUNTER_KERNEL_JSON_FAILED: {error}"))?;
    if json.len() > MAX_ALLOCATOR_COUNTER_KERNEL_JSON_BYTES {
        return Err("ALLOCATOR_COUNTER_KERNEL_JSON_TOO_LARGE".to_owned());
    }
    println!("{json}");
    Ok(())
}

fn parse_system_arguments(mut arguments: impl Iterator<Item = String>) -> Result<PathBuf, String> {
    let Some(flag) = arguments.next() else {
        return Err("allocator system helper requires --state-root <path>".to_owned());
    };
    if flag != "--state-root" {
        return Err(format!(
            "unexpected allocator system helper argument: {flag}"
        ));
    }
    let state_root = arguments
        .next()
        .ok_or_else(|| "--state-root requires a path".to_owned())?;
    if let Some(extra) = arguments.next() {
        return Err(format!(
            "unexpected allocator system helper argument: {extra}"
        ));
    }
    Ok(PathBuf::from(state_root))
}

fn parse_instrumented_arguments(
    mut arguments: impl Iterator<Item = String>,
) -> Result<(AllocatorCounterKernelModeV1, PathBuf), String> {
    let mut mode = None;
    let mut state_root = None;
    while let Some(flag) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--mode" if mode.is_none() => {
                mode = Some(match value.as_str() {
                    "inactive" => AllocatorCounterKernelModeV1::InstrumentedInactive,
                    "enabled" => AllocatorCounterKernelModeV1::InstrumentedEnabled,
                    _ => {
                        return Err(format!(
                            "allocator instrumented helper mode must be inactive or enabled, got {value}"
                        ));
                    }
                });
            }
            "--state-root" if state_root.is_none() => state_root = Some(PathBuf::from(value)),
            "--mode" | "--state-root" => return Err(format!("duplicate argument: {flag}")),
            _ => {
                return Err(format!(
                    "unexpected allocator instrumented helper argument: {flag}"
                ));
            }
        }
    }
    Ok((
        mode.ok_or_else(|| "allocator instrumented helper requires --mode".to_owned())?,
        state_root
            .ok_or_else(|| "allocator instrumented helper requires --state-root".to_owned())?,
    ))
}

#[must_use]
pub const fn target_triple() -> &'static str {
    if cfg!(all(
        target_arch = "x86_64",
        target_os = "windows",
        target_env = "msvc"
    )) {
        "x86_64-pc-windows-msvc"
    } else if cfg!(all(target_arch = "x86_64", target_os = "linux")) {
        "x86_64-unknown-linux-gnu"
    } else if cfg!(all(target_arch = "aarch64", target_os = "macos")) {
        "aarch64-apple-darwin"
    } else {
        "unsupported-target"
    }
}
