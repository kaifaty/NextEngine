use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use super::*;

pub(super) fn record_performance_span(
    enabled: bool,
    spans: &mut Vec<xtask::performance::PerformanceSpanV1>,
    dropped_spans: &mut u64,
    category: &str,
    elapsed: Duration,
) -> u128 {
    if enabled {
        let started = Instant::now();
        if spans.len() < usize::try_from(xtask::performance::MAX_SPANS_PER_THREAD).unwrap_or(0) {
            spans.push(xtask::performance::PerformanceSpanV1 {
                category: category.to_owned(),
                thread_index: 0,
                duration_microseconds: u64::try_from(elapsed.as_micros()).unwrap_or(u64::MAX),
            });
        } else {
            *dropped_spans = dropped_spans.saturating_add(1);
        }
        started.elapsed().as_nanos()
    } else {
        0
    }
}

pub(super) fn smoke_metric(
    name: &str,
    elapsed_microseconds: u128,
) -> Result<xtask::performance::PerformanceMetricV1, String> {
    xtask::performance::PerformanceMetricV1::from_samples(
        name,
        "microseconds",
        vec![microseconds_u64(elapsed_microseconds)?],
        None,
    )
}

pub(super) fn microseconds_u64(value: u128) -> Result<u64, String> {
    u64::try_from(value).map_err(|error| format!("performance duration overflow: {error}"))
}

pub(super) fn read_performance_baseline(
    path: &Path,
) -> Result<xtask::performance::PerformanceBaselineV5, String> {
    const MAX_BASELINE_BYTES: u64 = 64 * 1024 * 1024;
    let metadata = fs::metadata(path)
        .map_err(|error| format!("failed to inspect baseline {}: {error}", path.display()))?;
    if !metadata.is_file() || metadata.len() > MAX_BASELINE_BYTES {
        return Err(format!(
            "performance baseline must be a regular file no larger than {MAX_BASELINE_BYTES} bytes"
        ));
    }
    let bytes = fs::read(path)
        .map_err(|error| format!("failed to read baseline {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| format!("invalid PerformanceBaselineV5: {error}"))
}

pub(super) fn performance_command_report(
    run: xtask::performance::PerformanceRunV5,
    streaming: Option<StreamingPerformanceDetailsV1>,
    agent_planning: Option<AgentPerformanceDetailsV1>,
    render_planning: Option<RenderPlanningPerformanceDetailsV1>,
    live_runtime: Option<LiveRuntimePerformanceDetailsV1>,
    production_worker: Option<ProductionWorkerPerformanceDetailsV1>,
) -> CommandReportV1<PerformanceDetailsV1> {
    let status = run.verdict.command_report_status();
    CommandReportV1::new(
        "performance",
        status,
        PerformanceDetailsV1 {
            run: Some(run),
            streaming,
            agent_planning,
            render_planning,
            live_runtime,
            production_worker,
            r5_physics: None,
        },
    )
}

pub(super) fn write_performance_report(output: &Path, bytes: &[u8]) -> Result<(), String> {
    fs::create_dir_all(output).map_err(|error| {
        format!(
            "failed to create performance output {}: {error}",
            output.display()
        )
    })?;
    let final_path = output.join(xtask::performance::PERFORMANCE_REPORT_FILE_NAME);
    let temporary_path = output.join(xtask::performance::PERFORMANCE_REPORT_TEMP_FILE_NAME);
    if final_path.exists() || temporary_path.exists() {
        return Err(format!(
            "performance output already exists: {}",
            final_path.display()
        ));
    }
    fs::write(&temporary_path, bytes).map_err(|error| {
        format!(
            "failed to write performance report {}: {error}",
            temporary_path.display()
        )
    })?;
    fs::rename(&temporary_path, &final_path).map_err(|error| {
        format!(
            "failed to publish performance report {}: {error}",
            final_path.display()
        )
    })
}
