#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use xtask::performance::performance_scenario_hash;
use xtask::report::*;

use crate::run_tool_session;

mod support;

use support::*;

const INTERACTIVE_FRAME_SOAK_FRAMES: u32 = 240;
mod production_worker;
mod r2_alpha_render;
mod r3_multiregion;
mod r5_physics;
mod workloads;

use production_worker::*;
use workloads::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PerformanceArguments {
    scenario: xtask::performance::PerformanceScenarioV1,
    mode: xtask::performance::PerformanceModeV1,
    target: Option<String>,
    baseline: Option<PathBuf>,
    output: Option<PathBuf>,
    require_ready_preflight: bool,
}

impl Default for PerformanceArguments {
    fn default() -> Self {
        Self {
            scenario: xtask::performance::PerformanceScenarioV1::Smoke,
            mode: xtask::performance::PerformanceModeV1::Report,
            target: None,
            baseline: None,
            output: None,
            require_ready_preflight: false,
        }
    }
}

fn preserve_report_only_scenario_verdict(
    scenario: xtask::performance::PerformanceScenarioV1,
    mode: xtask::performance::PerformanceModeV1,
    verdict: xtask::performance::PerformanceVerdict,
) -> xtask::performance::PerformanceVerdict {
    let report_only_scenario = matches!(
        scenario,
        xtask::performance::PerformanceScenarioV1::Smoke
            | xtask::performance::PerformanceScenarioV1::LongSessionSoak
            | xtask::performance::PerformanceScenarioV1::InteractiveFrameSoak
            | xtask::performance::PerformanceScenarioV1::ProductionWorkerSoak
            | xtask::performance::PerformanceScenarioV1::R2AlphaRender
            | xtask::performance::PerformanceScenarioV1::R3MultiregionStreaming
    );
    if mode == xtask::performance::PerformanceModeV1::Report
        && report_only_scenario
        && verdict != xtask::performance::PerformanceVerdict::NotRun
    {
        xtask::performance::PerformanceVerdict::ReportOnly
    } else {
        verdict
    }
}

fn report_only_gate_diagnostic(
    scenario: xtask::performance::PerformanceScenarioV1,
) -> Option<&'static str> {
    match scenario {
        xtask::performance::PerformanceScenarioV1::Smoke => Some(
            "PERF_SMOKE_REPORT_ONLY: smoke fixtures have no hard timing budget and cannot gate",
        ),
        xtask::performance::PerformanceScenarioV1::LongSessionSoak => Some(
            "PERF_LONG_SESSION_SOAK_REPORT_ONLY: the long-session soak has no hard timing budget and cannot gate",
        ),
        xtask::performance::PerformanceScenarioV1::InteractiveFrameSoak => Some(
            "PERF_INTERACTIVE_FRAME_SOAK_REPORT_ONLY: the interactive frame soak is diagnostic and cannot gate",
        ),
        xtask::performance::PerformanceScenarioV1::ProductionWorkerSoak => Some(
            "PERF_PRODUCTION_WORKER_SOAK_REPORT_ONLY: the production worker soak is diagnostic and cannot gate",
        ),
        xtask::performance::PerformanceScenarioV1::R3MultiregionStreaming => Some(
            "PERF_R3_MULTIREGION_STREAMING_REPORT_ONLY: B-12 and the clean ten-run THOTH hard gate remain open",
        ),
        _ => None,
    }
}

pub(crate) fn parse_arguments(
    mut arguments: impl Iterator<Item = String>,
) -> Result<PerformanceArguments, String> {
    let mut request = PerformanceArguments::default();
    let mut scenario_seen = false;
    let mut mode_seen = false;
    while let Some(flag) = arguments.next() {
        if flag == "--require-ready-preflight" {
            if request.require_ready_preflight {
                return Err(format!("duplicate argument: {flag}"));
            }
            request.require_ready_preflight = true;
            continue;
        }
        let value = arguments
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--scenario" if !scenario_seen => {
                request.scenario = xtask::performance::PerformanceScenarioV1::parse(&value)?;
                scenario_seen = true;
            }
            "--mode" if !mode_seen => {
                request.mode = xtask::performance::PerformanceModeV1::parse(&value)?;
                mode_seen = true;
            }
            "--target" if request.target.is_none() => request.target = Some(value),
            "--baseline" if request.baseline.is_none() => {
                request.baseline = Some(PathBuf::from(value));
            }
            "--output" if request.output.is_none() => {
                request.output = Some(PathBuf::from(value));
            }
            "--scenario" | "--mode" | "--target" | "--baseline" | "--output" => {
                return Err(format!("duplicate argument: {flag}"));
            }
            _ => return Err(format!("unexpected argument: {flag}")),
        }
    }
    if request.mode == xtask::performance::PerformanceModeV1::Gate && request.target.is_none() {
        request.target = Some(xtask::performance::THOTH_TARGET_ID.to_owned());
    }
    Ok(request)
}

pub(crate) fn performance(root: &Path, request: &PerformanceArguments) -> Result<(), String> {
    let report = performance_report_for(root, request, None)?;
    let serialized = report.to_json()?;
    println!("{serialized}");
    if let Some(output) = &request.output {
        write_performance_report(output, serialized.as_bytes())?;
    }
    if report
        .details
        .run
        .as_ref()
        .is_some_and(|run| run.verdict == xtask::performance::PerformanceVerdict::Fail)
    {
        Err("PERFORMANCE_GATE_FAILED: hard performance verdict is FAIL".to_owned())
    } else {
        Ok(())
    }
}

pub(crate) fn performance_report(
    root: &Path,
    state_root: Option<&Path>,
) -> Result<CommandReportV1<PerformanceDetailsV1>, String> {
    performance_report_for(root, &PerformanceArguments::default(), state_root)
}

fn performance_report_for(
    root: &Path,
    request: &PerformanceArguments,
    state_root: Option<&Path>,
) -> Result<CommandReportV1<PerformanceDetailsV1>, String> {
    if request.mode == xtask::performance::PerformanceModeV1::Gate {
        return performance_gate_batch_report_for(root, request, state_root);
    }
    let mut report = performance_report_once(root, request, state_root, true)?;
    append_environment_postflight(request, &mut report, request.require_ready_preflight);
    Ok(report)
}

fn performance_gate_batch_report_for(
    root: &Path,
    request: &PerformanceArguments,
    state_root: Option<&Path>,
) -> Result<CommandReportV1<PerformanceDetailsV1>, String> {
    let mut reports = Vec::with_capacity(
        usize::try_from(xtask::performance::HARD_GATE_EVIDENCE_RUNS)
            .map_err(|error| error.to_string())?,
    );
    for _ in 0..xtask::performance::HARD_GATE_EVIDENCE_RUNS {
        let mut report = performance_report_once(root, request, state_root, false)?;
        append_environment_postflight(request, &mut report, true);
        let Some(run) = report.details.run.as_mut() else {
            return Err("PERF_GATE_RUN_EVIDENCE_MISSING".to_owned());
        };
        if run.verdict == xtask::performance::PerformanceVerdict::NotRun {
            run.diagnostics
                .push("PERF_GATE_BATCH_INCOMPLETE".to_owned());
            run.diagnostics.sort();
            run.diagnostics.dedup();
            report.status = run.verdict.command_report_status().to_owned();
            return Ok(report);
        }
        reports.push(report);
    }
    aggregate_gate_batch(request, reports)
}

fn append_environment_postflight(
    request: &PerformanceArguments,
    report: &mut CommandReportV1<PerformanceDetailsV1>,
    enforce_ready: bool,
) {
    let Some(run) = report.details.run.as_mut() else {
        return;
    };
    let target_id = request.target.as_deref().unwrap_or("observed-host-v1");
    match xtask::performance::inspect_current_host(target_id) {
        Ok((fingerprint, postflight)) => {
            if run.target_fingerprint.as_ref() != Some(&fingerprint) {
                run.diagnostics
                    .push("PERF_POSTFLIGHT_FINGERPRINT_MISMATCH".to_owned());
            }
            if enforce_ready && let Err(diagnostics) = postflight.validate_postflight_evidence() {
                run.diagnostics.extend(diagnostics);
            }
            run.environment_samples.push(postflight);
        }
        Err(error) if enforce_ready => run.diagnostics.push(error),
        Err(_) => {}
    }
    if !run.diagnostics.is_empty() {
        run.diagnostics.sort();
        run.diagnostics.dedup();
        run.verdict = xtask::performance::PerformanceVerdict::NotRun;
    }
    report.status = run.verdict.command_report_status().to_owned();
}

fn aggregate_gate_batch(
    request: &PerformanceArguments,
    mut reports: Vec<CommandReportV1<PerformanceDetailsV1>>,
) -> Result<CommandReportV1<PerformanceDetailsV1>, String> {
    if reports.len()
        != usize::try_from(xtask::performance::HARD_GATE_EVIDENCE_RUNS)
            .map_err(|error| error.to_string())?
    {
        return Err("PERF_GATE_BATCH_SIZE_INVALID".to_owned());
    }
    let runs = reports
        .iter()
        .map(|report| {
            report
                .details
                .run
                .as_ref()
                .ok_or_else(|| "PERF_GATE_RUN_EVIDENCE_MISSING".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let r5_details = aggregate_r5_details(&reports)?;
    let first = runs[0];
    let expected_metric_shape = first
        .metrics
        .iter()
        .map(|metric| (&metric.name, &metric.unit, &metric.absolute_budget))
        .collect::<Vec<_>>();
    let mut diagnostics = Vec::new();
    for (index, run) in runs.iter().enumerate().skip(1) {
        if run.schema_version != first.schema_version
            || run.commit != first.commit
            || run.worktree_clean != first.worktree_clean
            || run.toolchain != first.toolchain
            || run.target_triple != first.target_triple
            || run.scenario != first.scenario
            || run.scenario_hash != first.scenario_hash
            || run.content_hash != first.content_hash
            || run.target_fingerprint != first.target_fingerprint
            || run.build_profile != first.build_profile
            || run.mode != first.mode
            || run.methodology != first.methodology
            || run.authoritative_hashes != first.authoritative_hashes
            || run
                .metrics
                .iter()
                .map(|metric| (&metric.name, &metric.unit, &metric.absolute_budget))
                .collect::<Vec<_>>()
                != expected_metric_shape
        {
            diagnostics.push(format!("PERF_GATE_BATCH_RUN_INCOMPATIBLE: {index}"));
        }
    }
    let mut metrics = Vec::with_capacity(first.metrics.len());
    for expected in &first.metrics {
        let mut sample_runs = Vec::with_capacity(runs.len());
        for (index, run) in runs.iter().enumerate() {
            let Some(metric) = run
                .metrics
                .iter()
                .find(|metric| metric.name == expected.name)
            else {
                diagnostics.push(format!(
                    "PERF_GATE_BATCH_METRIC_MISSING: {index}: {}",
                    expected.name
                ));
                continue;
            };
            if metric.unit != expected.unit || metric.absolute_budget != expected.absolute_budget {
                diagnostics.push(format!(
                    "PERF_GATE_BATCH_METRIC_INCOMPATIBLE: {index}: {}",
                    expected.name
                ));
            }
            sample_runs.push(metric.raw_samples.clone());
        }
        if sample_runs.len() == runs.len() {
            metrics.push(xtask::performance::PerformanceMetricV1::from_sample_runs(
                expected.name.clone(),
                expected.unit.clone(),
                sample_runs,
                expected.absolute_budget.clone(),
            )?);
        }
    }
    let mut batch_run = first.clone();
    batch_run.evidence_runs = xtask::performance::HARD_GATE_EVIDENCE_RUNS;
    batch_run.environment_samples = runs
        .iter()
        .flat_map(|run| run.environment_samples.iter().cloned())
        .collect();
    batch_run.metrics = metrics;
    batch_run.instrumentation = aggregate_instrumentation(&runs)?;
    batch_run.resource_counters = aggregate_resource_counters(&runs)?;
    batch_run.diagnostics = diagnostics;
    batch_run.verdict = xtask::performance::aggregate_metric_verdict(&batch_run.metrics);
    if let Err(errors) = batch_run.validate_hard_evidence() {
        batch_run.diagnostics.extend(errors);
        batch_run.verdict = xtask::performance::PerformanceVerdict::NotRun;
    }
    if batch_run.verdict != xtask::performance::PerformanceVerdict::NotRun {
        let baseline_path = request
            .baseline
            .as_ref()
            .ok_or_else(|| "PERF_GATE_BASELINE_REQUIRED".to_owned())?;
        match read_performance_baseline(baseline_path) {
            Ok(baseline) => {
                if let Err(errors) =
                    xtask::performance::compare_metrics_to_baseline(&mut batch_run, &baseline)
                {
                    batch_run.diagnostics.extend(errors);
                    batch_run.verdict = xtask::performance::PerformanceVerdict::NotRun;
                }
            }
            Err(error) => {
                batch_run
                    .diagnostics
                    .push(format!("PERF_BASELINE_INVALID: {error}"));
                batch_run.verdict = xtask::performance::PerformanceVerdict::NotRun;
            }
        }
    }
    batch_run.diagnostics.sort();
    batch_run.diagnostics.dedup();
    let status = batch_run.verdict.command_report_status().to_owned();
    let mut report = reports.remove(0);
    if r5_details.is_some() {
        report.details.r5_physics = r5_details;
    }
    report.details.run = Some(batch_run);
    report.status = status;
    Ok(report)
}

fn aggregate_r5_details(
    reports: &[CommandReportV1<PerformanceDetailsV1>],
) -> Result<Option<R5PhysicsPerformanceDetailsV1>, String> {
    let details = reports
        .iter()
        .map(|report| report.details.r5_physics.as_ref())
        .collect::<Vec<_>>();
    if details.iter().all(|details| details.is_none()) {
        return Ok(None);
    }
    let details = details
        .into_iter()
        .map(|details| details.ok_or_else(|| "PERF_GATE_BATCH_R5_DETAILS_MISSING".to_owned()))
        .collect::<Result<Vec<_>, _>>()?;
    let first = details[0];
    for (index, detail) in details.iter().enumerate().skip(1) {
        if detail.slot_count != first.slot_count
            || detail.degrees_of_freedom_per_slot != first.degrees_of_freedom_per_slot
            || detail.physics_hz != first.physics_hz
            || detail.motor_hz != first.motor_hz
            || detail.warmup_substeps_per_slot != first.warmup_substeps_per_slot
            || detail.measured_substeps_per_slot != first.measured_substeps_per_slot
            || detail.measured_motor_frames_per_slot != first.measured_motor_frames_per_slot
            || detail.replay_prefix_substeps_per_slot != first.replay_prefix_substeps_per_slot
            || detail.authoritative_root != first.authoritative_root
            || !detail.worker_root_parity
            || detail.worker_runs.len() != first.worker_runs.len()
        {
            return Err(format!("PERF_GATE_BATCH_R5_DETAILS_INCOMPATIBLE: {index}"));
        }
    }
    let mut worker_runs = Vec::with_capacity(first.worker_runs.len());
    for expected in &first.worker_runs {
        let workers = details
            .iter()
            .map(|detail| {
                detail
                    .worker_runs
                    .iter()
                    .find(|worker| worker.worker_count == expected.worker_count)
                    .ok_or_else(|| {
                        format!(
                            "PERF_GATE_BATCH_R5_WORKER_DETAILS_MISSING: {}",
                            expected.worker_count
                        )
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        if workers
            .iter()
            .any(|worker| worker.authoritative_root != expected.authoritative_root)
        {
            return Err(format!(
                "PERF_GATE_BATCH_R5_WORKER_ROOT_DIVERGED: {}",
                expected.worker_count
            ));
        }
        worker_runs.push(R5PhysicsWorkerPerformanceDetailsV1 {
            worker_count: expected.worker_count,
            elapsed_microseconds: median_u64(
                workers.iter().map(|worker| worker.elapsed_microseconds),
            )?,
            aggregate_physics_substeps_per_second: median_u64(
                workers
                    .iter()
                    .map(|worker| worker.aggregate_physics_substeps_per_second),
            )?,
            aggregate_motor_frames_per_second: median_u64(
                workers
                    .iter()
                    .map(|worker| worker.aggregate_motor_frames_per_second),
            )?,
            scaling_efficiency_basis_points: median_u64(
                workers
                    .iter()
                    .map(|worker| worker.scaling_efficiency_basis_points),
            )?,
            motor_frame_p95_microseconds: workers
                .iter()
                .map(|worker| worker.motor_frame_p95_microseconds)
                .max()
                .unwrap_or(0),
            motor_frame_p99_microseconds: workers
                .iter()
                .map(|worker| worker.motor_frame_p99_microseconds)
                .max()
                .unwrap_or(0),
            authoritative_root: expected.authoritative_root.clone(),
        });
    }
    let checkpoint_bytes_per_slot = details
        .iter()
        .flat_map(|detail| detail.checkpoint_bytes_per_slot.iter().copied())
        .collect();
    let restore_microseconds_per_slot = details
        .iter()
        .flat_map(|detail| detail.restore_microseconds_per_slot.iter().copied())
        .collect();
    Ok(Some(R5PhysicsPerformanceDetailsV1 {
        evidence_run_count: xtask::performance::HARD_GATE_EVIDENCE_RUNS,
        slot_count: first.slot_count,
        degrees_of_freedom_per_slot: first.degrees_of_freedom_per_slot,
        physics_hz: first.physics_hz,
        motor_hz: first.motor_hz,
        warmup_substeps_per_slot: first.warmup_substeps_per_slot,
        measured_substeps_per_slot: first.measured_substeps_per_slot,
        measured_motor_frames_per_slot: first.measured_motor_frames_per_slot,
        worker_runs,
        checkpoint_bytes_per_slot,
        restore_microseconds_per_slot,
        restore_p95_microseconds: details
            .iter()
            .map(|detail| detail.restore_p95_microseconds)
            .max()
            .unwrap_or(0),
        restore_p99_microseconds: details
            .iter()
            .map(|detail| detail.restore_p99_microseconds)
            .max()
            .unwrap_or(0),
        restore_wall_microseconds: median_u64(
            details
                .iter()
                .map(|detail| detail.restore_wall_microseconds),
        )?,
        replay_prefix_substeps_per_slot: first.replay_prefix_substeps_per_slot,
        replay_prefix_overhead_basis_points: details
            .iter()
            .map(|detail| detail.replay_prefix_overhead_basis_points)
            .max()
            .unwrap_or(0),
        process_peak_working_set_bytes: details
            .iter()
            .map(|detail| detail.process_peak_working_set_bytes)
            .max()
            .unwrap_or(0),
        logical_host_bytes_per_slot: details
            .iter()
            .map(|detail| detail.logical_host_bytes_per_slot)
            .max()
            .unwrap_or(0),
        authoritative_root: first.authoritative_root.clone(),
        worker_root_parity: true,
    }))
}

fn median_u64(values: impl Iterator<Item = u64>) -> Result<u64, String> {
    xtask::performance::nearest_rank_percentile(&values.collect::<Vec<_>>(), 50)
}

fn aggregate_instrumentation(
    runs: &[&xtask::performance::PerformanceRunV5],
) -> Result<xtask::performance::PerformanceInstrumentationV1, String> {
    let mut recorded_spans = Vec::new();
    let mut thread_offset = 0_u32;
    let mut max_spans_per_thread = 0_u32;
    let mut reserved_bytes = 0_u64;
    let mut dropped_spans = 0_u64;
    let mut unowned_spans = 0_u64;
    let mut overhead_basis_points = Some(i64::MIN);
    let mut authoritative_hash_parity = Some(true);
    for run in runs {
        let instrumentation = &run.instrumentation;
        for span in &instrumentation.recorded_spans {
            let mut span = span.clone();
            span.thread_index = span
                .thread_index
                .checked_add(thread_offset)
                .ok_or_else(|| "PERF_PROFILER_THREAD_COUNT_OVERFLOW".to_owned())?;
            recorded_spans.push(span);
        }
        thread_offset = thread_offset
            .checked_add(instrumentation.max_threads)
            .ok_or_else(|| "PERF_PROFILER_THREAD_COUNT_OVERFLOW".to_owned())?;
        max_spans_per_thread = max_spans_per_thread.max(instrumentation.max_spans_per_thread);
        reserved_bytes = reserved_bytes
            .checked_add(instrumentation.reserved_bytes)
            .ok_or_else(|| "PERF_PROFILER_MEMORY_OVERFLOW".to_owned())?;
        dropped_spans = dropped_spans
            .checked_add(instrumentation.dropped_spans)
            .ok_or_else(|| "PERF_PROFILER_SPAN_COUNT_OVERFLOW".to_owned())?;
        unowned_spans = unowned_spans
            .checked_add(instrumentation.unowned_spans)
            .ok_or_else(|| "PERF_PROFILER_SPAN_COUNT_OVERFLOW".to_owned())?;
        overhead_basis_points = match (overhead_basis_points, instrumentation.overhead_basis_points)
        {
            (Some(current), Some(value)) => Some(current.max(value)),
            _ => None,
        };
        authoritative_hash_parity = match (
            authoritative_hash_parity,
            instrumentation.authoritative_hash_parity,
        ) {
            (Some(current), Some(value)) => Some(current && value),
            _ => None,
        };
    }
    Ok(xtask::performance::PerformanceInstrumentationV1 {
        enabled: runs.iter().all(|run| run.instrumentation.enabled),
        max_threads: thread_offset,
        max_spans_per_thread,
        reserved_bytes,
        recorded_spans,
        dropped_spans,
        unowned_spans,
        overhead_basis_points,
        authoritative_hash_parity,
    })
}

fn aggregate_resource_counters(
    runs: &[&xtask::performance::PerformanceRunV5],
) -> Result<xtask::performance::PerformanceResourceCountersV4, String> {
    let first = &runs[0].resource_counters;
    let mut unavailable = BTreeSet::new();
    let mut io_read_bytes = 0_u64;
    let mut io_write_bytes = 0_u64;
    let mut vulkan_timestamp_queries = 0_u64;
    for run in runs {
        let counters = &run.resource_counters;
        if counters.logical_resource_charges != first.logical_resource_charges {
            return Err("PERF_GATE_BATCH_LOGICAL_CHARGES_DIVERGED".to_owned());
        }
        unavailable.extend(counters.unavailable.iter().cloned());
        io_read_bytes = io_read_bytes
            .checked_add(
                counters
                    .io_read_bytes
                    .ok_or_else(|| "PERF_REQUIRED_COUNTER_MISSING: io_read_bytes".to_owned())?,
            )
            .ok_or_else(|| "PERF_RESOURCE_COUNTER_OVERFLOW".to_owned())?;
        io_write_bytes = io_write_bytes
            .checked_add(
                counters
                    .io_write_bytes
                    .ok_or_else(|| "PERF_REQUIRED_COUNTER_MISSING: io_write_bytes".to_owned())?,
            )
            .ok_or_else(|| "PERF_RESOURCE_COUNTER_OVERFLOW".to_owned())?;
        vulkan_timestamp_queries = vulkan_timestamp_queries
            .checked_add(counters.vulkan_timestamp_queries)
            .ok_or_else(|| "PERF_RESOURCE_COUNTER_OVERFLOW".to_owned())?;
    }
    Ok(xtask::performance::PerformanceResourceCountersV4 {
        process_peak_working_set_bytes: runs
            .iter()
            .filter_map(|run| run.resource_counters.process_peak_working_set_bytes)
            .max(),
        device_resident_bytes: runs
            .iter()
            .filter_map(|run| run.resource_counters.device_resident_bytes)
            .max(),
        io_read_bytes: Some(io_read_bytes),
        io_write_bytes: Some(io_write_bytes),
        logical_resource_charges: first.logical_resource_charges.clone(),
        vulkan_timestamp_queries,
        unavailable: unavailable.into_iter().collect(),
    })
}

fn performance_report_once(
    root: &Path,
    request: &PerformanceArguments,
    state_root: Option<&Path>,
    compare_baseline: bool,
) -> Result<CommandReportV1<PerformanceDetailsV1>, String> {
    let mut run = xtask::performance::PerformanceRunV5::empty(
        request.scenario,
        request.mode,
        env!("NEXTENGINE_BUILD_PROFILE"),
    );
    run.scenario_hash = performance_scenario_hash(request.scenario);
    populate_performance_identity(root, &mut run);
    populate_performance_host(request, &mut run);

    if request.require_ready_preflight {
        let mut preflight_diagnostics = match &run.preflight {
            Some(preflight) => preflight
                .validate_ready_evidence()
                .err()
                .unwrap_or_default(),
            None => vec!["PERF_PREFLIGHT_UNAVAILABLE".to_owned()],
        };
        if !preflight_diagnostics.is_empty() || !run.diagnostics.is_empty() {
            run.diagnostics.append(&mut preflight_diagnostics);
            run.diagnostics.sort();
            run.diagnostics.dedup();
            run.verdict = xtask::performance::PerformanceVerdict::NotRun;
            return Ok(performance_command_report(
                run, None, None, None, None, None,
            ));
        }
    }

    if request.mode == xtask::performance::PerformanceModeV1::Gate {
        if let Some(diagnostic) = report_only_gate_diagnostic(request.scenario) {
            run.diagnostics.push(diagnostic.to_owned());
            run.verdict = xtask::performance::PerformanceVerdict::NotRun;
            return Ok(performance_command_report(
                run, None, None, None, None, None,
            ));
        }
        validate_gate_prerequisites(request, &mut run);
        if !run.diagnostics.is_empty() {
            run.verdict = xtask::performance::PerformanceVerdict::NotRun;
            return Ok(performance_command_report(
                run, None, None, None, None, None,
            ));
        }
    }

    if let Some(reason) = request.scenario.unavailable_reason() {
        run.diagnostics.push(reason.to_owned());
        run.verdict = xtask::performance::PerformanceVerdict::NotRun;
        return Ok(performance_command_report(
            run, None, None, None, None, None,
        ));
    }
    let (tool_run, _) = run_tool_session("tools-performance-v2", state_root)?;
    let profiling_enabled = match env::var("NEXTENGINE_PERFORMANCE_PROFILER") {
        Ok(value) if value.eq_ignore_ascii_case("on") || value == "1" => true,
        Ok(value) if value.eq_ignore_ascii_case("off") || value == "0" => false,
        Ok(value) => {
            return Err(format!(
                "NEXTENGINE_PERFORMANCE_PROFILER must be on/off or 1/0, got {value}"
            ));
        }
        Err(env::VarError::NotPresent) => false,
        Err(error) => return Err(error.to_string()),
    };
    if request.scenario == xtask::performance::PerformanceScenarioV1::R3MultiregionStreaming {
        return r3_multiregion::performance_report(
            request,
            state_root,
            run,
            tool_run.project_composition_lock_hash.to_hex(),
            profiling_enabled,
            compare_baseline,
        );
    }
    if request.scenario == xtask::performance::PerformanceScenarioV1::R5Physics16 {
        return r5_physics::performance_report(
            request,
            run,
            tool_run.project_composition_lock_hash.to_hex(),
            profiling_enabled,
            compare_baseline,
        );
    }
    let mut recorded_spans = if profiling_enabled {
        Vec::with_capacity(
            usize::try_from(xtask::performance::MAX_SPANS_PER_THREAD)
                .map_err(|error| error.to_string())?,
        )
    } else {
        Vec::new()
    };
    let mut dropped_spans = 0_u64;
    let mut instrumentation_overhead_nanoseconds = 0_u128;
    let profiler_control = profiling_enabled
        .then(|| run_profiler_control(request.scenario, state_root))
        .transpose()?;
    let desktop_frame_timing_requested = profiling_enabled
        || request.scenario == xtask::performance::PerformanceScenarioV1::InteractiveFrameSoak;
    let ScenarioWorkloads {
        streaming,
        agent,
        render_planning,
        live_runtime,
        production_worker,
        desktop_frame_timing,
        r2_alpha_render,
        resource_counters,
    } = run_scenario_workloads(
        request.scenario,
        state_root,
        desktop_frame_timing_requested,
        INTERACTIVE_FRAME_SOAK_FRAMES,
    )?;

    instrumentation_overhead_nanoseconds =
        instrumentation_overhead_nanoseconds.saturating_add(record_performance_span(
            profiling_enabled,
            &mut recorded_spans,
            &mut dropped_spans,
            "streaming-io",
            streaming.elapsed,
        ));
    instrumentation_overhead_nanoseconds =
        instrumentation_overhead_nanoseconds.saturating_add(record_performance_span(
            profiling_enabled,
            &mut recorded_spans,
            &mut dropped_spans,
            "agent-planning",
            agent.elapsed,
        ));
    instrumentation_overhead_nanoseconds =
        instrumentation_overhead_nanoseconds.saturating_add(record_performance_span(
            profiling_enabled,
            &mut recorded_spans,
            &mut dropped_spans,
            "render-extraction",
            render_planning.elapsed,
        ));
    instrumentation_overhead_nanoseconds =
        instrumentation_overhead_nanoseconds.saturating_add(record_performance_span(
            profiling_enabled,
            &mut recorded_spans,
            &mut dropped_spans,
            "runtime-stages",
            live_runtime.elapsed,
        ));
    if profiling_enabled && let Some(frame_timing) = &desktop_frame_timing {
        instrumentation_overhead_nanoseconds =
            instrumentation_overhead_nanoseconds.saturating_add(record_performance_span(
                profiling_enabled,
                &mut recorded_spans,
                &mut dropped_spans,
                "render-extraction",
                frame_timing.elapsed,
            ));
    }
    if profiling_enabled && let Some(r2_alpha_render) = &r2_alpha_render {
        instrumentation_overhead_nanoseconds =
            instrumentation_overhead_nanoseconds.saturating_add(record_performance_span(
                profiling_enabled,
                &mut recorded_spans,
                &mut dropped_spans,
                "render-extraction",
                r2_alpha_render.elapsed,
            ));
    }

    let streaming = streaming.report;
    let agent = agent.report;
    let render_planning = render_planning.report;
    let live_runtime = live_runtime.report;
    let production_worker = production_worker.map(|workload| workload.report);
    let desktop_frame_timing = desktop_frame_timing.and_then(|workload| workload.report);
    let r2_alpha_render = r2_alpha_render.map(|workload| workload.report);

    let mut authoritative_hashes = scenario_authoritative_hashes(
        &streaming,
        &agent,
        &render_planning,
        &live_runtime,
        production_worker.as_ref(),
    );
    if let Some(report) = &r2_alpha_render {
        r2_alpha_render::append_authoritative_hashes(&mut authoritative_hashes, report);
    }
    let worker_unowned_spans = production_worker
        .as_ref()
        .map(observed_worker_unowned_spans)
        .unwrap_or(0);
    let worker_reserved_bytes = production_worker
        .as_ref()
        .map(production_worker_reserved_bytes)
        .transpose()?
        .unwrap_or(0);
    if let Some(worker) = &production_worker {
        dropped_spans = dropped_spans.saturating_add(worker.metrics.dropped_callbacks);
        instrumentation_overhead_nanoseconds =
            instrumentation_overhead_nanoseconds.saturating_add(record_worker_owned_spans(
                profiling_enabled,
                &mut recorded_spans,
                &mut dropped_spans,
                worker,
            ));
    }

    let profiled_microseconds = recorded_spans
        .iter()
        .map(|span| span.duration_microseconds)
        .try_fold(0_u64, |total, value| total.checked_add(value))
        .ok_or_else(|| "profiled smoke duration overflow".to_owned())?;
    let (overhead_basis_points, authoritative_hash_parity) = profiler_control
        .as_ref()
        .map(|control| {
            (
                overhead_basis_points(instrumentation_overhead_nanoseconds, profiled_microseconds),
                control.authoritative_hashes == authoritative_hashes,
            )
        })
        .map_or((None, None), |(overhead, parity)| {
            (Some(overhead), Some(parity))
        });
    run.instrumentation = xtask::performance::PerformanceInstrumentationV1 {
        enabled: profiling_enabled,
        max_threads: if production_worker.is_some() { 2 } else { 1 },
        max_spans_per_thread: xtask::performance::MAX_SPANS_PER_THREAD,
        reserved_bytes: u64::try_from(
            recorded_spans.capacity()
                * std::mem::size_of::<xtask::performance::PerformanceSpanV1>(),
        )
        .map_err(|error| error.to_string())?
        .checked_add(worker_reserved_bytes)
        .ok_or_else(|| "performance instrumentation reservation overflow".to_owned())?,
        recorded_spans,
        dropped_spans,
        unowned_spans: worker_unowned_spans,
        overhead_basis_points,
        authoritative_hash_parity,
    };
    run.resource_counters = resource_counters;
    if request.scenario == xtask::performance::PerformanceScenarioV1::Smoke {
        run.resource_counters.logical_resource_charges = Some(
            xtask::performance::PerformanceLogicalResourceChargesV1::new(
                xtask::performance::sha256_hex(b"nextengine.performance.r3a-packaged-streaming.v1"),
                0,
                streaming.required_staging_bytes,
                0,
                0,
                0,
                0,
            )?,
        );
    }
    if let Some(frame_timing) = &desktop_frame_timing {
        run.resource_counters.vulkan_timestamp_queries = frame_timing.timestamp_query_count;
        run.resource_counters.device_resident_bytes = Some(frame_timing.device_allocation_bytes);
        run.resource_counters.unavailable.retain(|diagnostic| {
            !diagnostic.starts_with("Vulkan timestamps require")
                && !diagnostic.starts_with("device residency requires")
        });
    }
    if let Some(report) = &r2_alpha_render {
        r2_alpha_render::attach_resource_evidence(&mut run.resource_counters, report)?;
    }
    run.content_hash = tool_run.project_composition_lock_hash.to_hex();
    run.scenario_hash = performance_scenario_hash(request.scenario);
    let live_metric_prefix = match request.scenario {
        xtask::performance::PerformanceScenarioV1::Smoke => "smoke.live-runtime",
        xtask::performance::PerformanceScenarioV1::LongSessionSoak => {
            "long-session-soak.live-runtime"
        }
        xtask::performance::PerformanceScenarioV1::InteractiveFrameSoak => {
            "interactive-frame-soak.live-runtime-control"
        }
        xtask::performance::PerformanceScenarioV1::ProductionWorkerSoak => {
            "production-worker-soak.live-runtime-control"
        }
        xtask::performance::PerformanceScenarioV1::R2AlphaRender => {
            "r2-alpha-render.live-runtime-control"
        }
        _ => unreachable!("unavailable representative scenarios return before execution"),
    };
    run.metrics = vec![
        smoke_metric("smoke.streaming.total", streaming.elapsed_microseconds)?,
        smoke_metric("smoke.agent-planning.total", agent.elapsed_microseconds)?,
        smoke_metric(
            "smoke.render-planning.total",
            render_planning.elapsed_microseconds,
        )?,
        xtask::performance::PerformanceMetricV1::from_samples(
            format!("{live_metric_prefix}.window"),
            "microseconds",
            live_runtime
                .window_microseconds
                .into_iter()
                .map(microseconds_u64)
                .collect::<Result<Vec<_>, _>>()?,
            None,
        )?,
        xtask::performance::PerformanceMetricV1::from_samples(
            format!("{live_metric_prefix}.identity-index-root-probe"),
            "microseconds",
            live_runtime
                .identity_index_root_probe_microseconds
                .into_iter()
                .map(microseconds_u64)
                .collect::<Result<Vec<_>, _>>()?,
            None,
        )?,
        xtask::performance::PerformanceMetricV1::from_samples(
            format!("{live_metric_prefix}.archive-root-probe"),
            "microseconds",
            live_runtime
                .archive_root_probe_microseconds
                .into_iter()
                .map(microseconds_u64)
                .collect::<Result<Vec<_>, _>>()?,
            None,
        )?,
    ];
    if request.scenario == xtask::performance::PerformanceScenarioV1::LongSessionSoak {
        run.metrics
            .push(xtask::performance::PerformanceMetricV1::from_samples(
                "long-session-soak.live-runtime.driver-prepare",
                "microseconds",
                live_runtime
                    .driver_prepare_microseconds
                    .iter()
                    .copied()
                    .map(microseconds_u64)
                    .collect::<Result<Vec<_>, _>>()?,
                None,
            )?);
        run.metrics
            .push(xtask::performance::PerformanceMetricV1::from_samples(
                "long-session-soak.live-runtime.driver-commit",
                "microseconds",
                live_runtime
                    .driver_commit_microseconds
                    .iter()
                    .copied()
                    .map(microseconds_u64)
                    .collect::<Result<Vec<_>, _>>()?,
                None,
            )?);
        run.metrics
            .push(xtask::performance::PerformanceMetricV1::from_samples(
                "long-session-soak.live-runtime.checkpoint-materialization",
                "microseconds",
                live_runtime
                    .driver_checkpoint_materialization_microseconds
                    .iter()
                    .copied()
                    .map(microseconds_u64)
                    .collect::<Result<Vec<_>, _>>()?,
                None,
            )?);
        run.metrics
            .push(xtask::performance::PerformanceMetricV1::from_samples(
                "long-session-soak.application.window",
                "microseconds",
                live_runtime
                    .application_window_microseconds
                    .into_iter()
                    .map(microseconds_u64)
                    .collect::<Result<Vec<_>, _>>()?,
                None,
            )?);
        run.metrics
            .push(xtask::performance::PerformanceMetricV1::from_samples(
                "long-session-soak.application.sample-interval",
                "microseconds",
                live_runtime
                    .application_sample_interval_microseconds
                    .into_iter()
                    .map(microseconds_u64)
                    .collect::<Result<Vec<_>, _>>()?,
                None,
            )?);
        run.metrics
            .push(xtask::performance::PerformanceMetricV1::from_samples(
                "long-session-soak.application.non-sample-tick",
                "microseconds",
                live_runtime
                    .application_non_sample_tick_microseconds
                    .iter()
                    .copied()
                    .map(microseconds_u64)
                    .collect::<Result<Vec<_>, _>>()?,
                None,
            )?);
        run.metrics
            .push(xtask::performance::PerformanceMetricV1::from_samples(
                "long-session-soak.application.sample-tick",
                "microseconds",
                live_runtime
                    .application_sample_tick_microseconds
                    .iter()
                    .copied()
                    .map(microseconds_u64)
                    .collect::<Result<Vec<_>, _>>()?,
                None,
            )?);
    }
    if let Some(worker) = &production_worker {
        append_production_worker_metrics(&mut run.metrics, worker)?;
    }
    if let Some(frame_timing) = &desktop_frame_timing {
        let frame_prefix = if request.scenario
            == xtask::performance::PerformanceScenarioV1::InteractiveFrameSoak
        {
            "interactive-frame-soak.frame"
        } else {
            "smoke.frame"
        };
        let phase_samples = [
            (
                "cpu-extract-submit",
                frame_timing
                    .samples
                    .iter()
                    .map(|sample| sample.cpu_extract_and_submit_microseconds)
                    .collect(),
            ),
            (
                "gpu",
                frame_timing
                    .samples
                    .iter()
                    .map(|sample| sample.gpu_duration_microseconds)
                    .collect(),
            ),
            (
                "critical-path",
                frame_timing
                    .samples
                    .iter()
                    .map(|sample| {
                        sample
                            .cpu_extract_and_submit_microseconds
                            .max(sample.gpu_duration_microseconds)
                    })
                    .collect(),
            ),
            (
                "event-frame-source-update",
                frame_timing
                    .samples
                    .iter()
                    .map(|sample| sample.event_and_frame_source_update_microseconds)
                    .collect(),
            ),
            (
                "frame-slot-wait",
                frame_timing
                    .samples
                    .iter()
                    .map(|sample| sample.frame_slot_wait_microseconds)
                    .collect(),
            ),
            (
                "image-acquire-wait",
                frame_timing
                    .samples
                    .iter()
                    .map(|sample| sample.image_acquire_wait_microseconds)
                    .collect(),
            ),
            (
                "swapchain-image-wait",
                frame_timing
                    .samples
                    .iter()
                    .map(|sample| sample.swapchain_image_wait_microseconds)
                    .collect(),
            ),
            (
                "frame-plan",
                frame_timing
                    .samples
                    .iter()
                    .map(|sample| sample.frame_plan_microseconds)
                    .collect(),
            ),
            (
                "command-record",
                frame_timing
                    .samples
                    .iter()
                    .map(|sample| sample.command_record_microseconds)
                    .collect(),
            ),
            (
                "queue-submit",
                frame_timing
                    .samples
                    .iter()
                    .map(|sample| sample.queue_submit_microseconds)
                    .collect(),
            ),
            (
                "present-wait",
                frame_timing
                    .samples
                    .iter()
                    .map(|sample| sample.present_wait_microseconds)
                    .collect(),
            ),
        ];
        for (name, samples) in phase_samples {
            run.metrics
                .push(xtask::performance::PerformanceMetricV1::from_samples(
                    format!("{frame_prefix}.{name}"),
                    "microseconds",
                    samples,
                    None,
                )?);
        }
        let deadline_misses = u64::try_from(
            frame_timing
                .samples
                .iter()
                .filter(|sample| {
                    sample
                        .cpu_extract_and_submit_microseconds
                        .max(sample.gpu_duration_microseconds)
                        > 16_667
                })
                .count(),
        )
        .map_err(|error| error.to_string())?;
        for (name, unit, value) in [
            ("primary-deadline-misses", "count", deadline_misses),
            (
                "software-paced-iterations",
                "count",
                frame_timing.software_paced_iterations,
            ),
            (
                "software-pacing-sleep",
                "microseconds",
                frame_timing.software_pacing_sleep_microseconds,
            ),
            (
                "frame-plan-cache-hits",
                "count",
                frame_timing.frame_plan_cache_hits,
            ),
            (
                "frame-plan-cache-misses",
                "count",
                frame_timing.frame_plan_cache_misses,
            ),
            (
                "frame-plan-build-failures",
                "count",
                frame_timing.frame_plan_build_failures,
            ),
            (
                "frame-plan-explicit-invalidations",
                "count",
                frame_timing.frame_plan_explicit_invalidations,
            ),
        ] {
            run.metrics
                .push(xtask::performance::PerformanceMetricV1::from_samples(
                    format!("{frame_prefix}.{name}"),
                    unit,
                    vec![value],
                    None,
                )?);
        }
    }
    if let Some(report) = &r2_alpha_render {
        r2_alpha_render::append_metrics(&mut run.metrics, report)?;
    }
    run.authoritative_hashes = authoritative_hashes;
    run.verdict = xtask::performance::aggregate_metric_verdict(&run.metrics);
    if request.scenario == xtask::performance::PerformanceScenarioV1::InteractiveFrameSoak
        && desktop_frame_timing.is_none()
    {
        run.diagnostics.push(
            "PERF_INTERACTIVE_FRAME_TIMING_UNAVAILABLE: build xtask with --features desktop-sdl-ash on a supported desktop host"
                .to_owned(),
        );
        run.verdict = xtask::performance::PerformanceVerdict::NotRun;
    }
    if request.scenario == xtask::performance::PerformanceScenarioV1::R2AlphaRender
        && r2_alpha_render.is_none()
    {
        run.diagnostics.push(
            "PERF_R2_ALPHA_RENDER_UNAVAILABLE: build xtask with --features desktop-sdl-ash on a supported Windows desktop host"
                .to_owned(),
        );
        run.verdict = xtask::performance::PerformanceVerdict::NotRun;
    }
    if let Err(error) = run.instrumentation.validate() {
        run.diagnostics.push(error);
        run.verdict = xtask::performance::PerformanceVerdict::NotRun;
    }
    let evidence_validation = if request.mode == xtask::performance::PerformanceModeV1::Gate {
        run.validate_hard_evidence()
    } else {
        run.validate_report_evidence()
    };
    if let Err(diagnostics) = evidence_validation {
        run.diagnostics.extend(diagnostics);
        run.verdict = xtask::performance::PerformanceVerdict::NotRun;
    }
    if !run.diagnostics.is_empty() {
        run.verdict = xtask::performance::PerformanceVerdict::NotRun;
    }

    if compare_baseline
        && run.verdict != xtask::performance::PerformanceVerdict::NotRun
        && let Some(path) = &request.baseline
    {
        match read_performance_baseline(path) {
            Ok(baseline) => {
                if let Err(diagnostics) =
                    xtask::performance::compare_metrics_to_baseline(&mut run, &baseline)
                {
                    run.diagnostics.extend(diagnostics);
                    run.verdict = xtask::performance::PerformanceVerdict::NotRun;
                }
            }
            Err(error) => {
                run.diagnostics
                    .push(format!("PERF_BASELINE_INVALID: {error}"));
                run.verdict = xtask::performance::PerformanceVerdict::NotRun;
            }
        }
    }
    run.verdict =
        preserve_report_only_scenario_verdict(request.scenario, request.mode, run.verdict);

    let production_worker_details = production_worker
        .as_ref()
        .map(production_worker_details)
        .transpose()?;
    Ok(performance_command_report(
        run,
        Some(StreamingPerformanceDetailsV1 {
            cycles: streaming.cycles,
            staged_asset_references: streaming.staged_asset_references,
            elapsed_microseconds: streaming.elapsed_microseconds,
            final_generation: streaming.final_generation,
            final_world_state_hash: streaming.final_world_state_hash.to_hex(),
        }),
        Some(AgentPerformanceDetailsV1 {
            cycles: agent.cycles,
            elapsed_microseconds: agent.elapsed_microseconds,
            final_plan_hash: agent.final_plan_hash.to_hex(),
        }),
        Some(RenderPlanningPerformanceDetailsV1 {
            cycles: render_planning.cycles,
            elapsed_microseconds: render_planning.elapsed_microseconds,
            visible_object_count: render_planning.visible_object_count,
            indexed_draw_count: render_planning.indexed_draw_count,
            fallback_material_draw_count: render_planning.fallback_material_draw_count,
            frame_plan_hash: render_planning.frame_plan_hash.to_hex(),
        }),
        Some(LiveRuntimePerformanceDetailsV1 {
            ticks: live_runtime.ticks,
            command_body_count: live_runtime.command_body_count,
            elapsed_microseconds: live_runtime.elapsed_microseconds,
            window_microseconds: live_runtime.window_microseconds,
            checkpoint_microseconds: live_runtime.checkpoint_microseconds,
            final_state_root: live_runtime.final_state_root.to_hex(),
        }),
        production_worker_details,
    ))
}

fn overhead_basis_points(overhead_nanoseconds: u128, workload_microseconds: u64) -> i64 {
    if workload_microseconds == 0 {
        return if overhead_nanoseconds == 0 {
            0
        } else {
            i64::MAX
        };
    }
    let value = i128::try_from(overhead_nanoseconds)
        .unwrap_or(i128::MAX)
        .saturating_mul(10_000)
        / i128::from(workload_microseconds).saturating_mul(1_000);
    i64::try_from(value).unwrap_or(i64::MAX)
}

fn populate_performance_identity(root: &Path, run: &mut xtask::performance::PerformanceRunV5) {
    run.commit = env!("NEXTENGINE_BUILD_COMMIT").to_owned();
    run.worktree_clean = match env!("NEXTENGINE_BUILD_WORKTREE_CLEAN") {
        "true" => true,
        "false" => false,
        value => {
            run.diagnostics.push(format!(
                "PERF_BUILD_WORKTREE_STATE_INVALID: compile-time value {value:?}"
            ));
            false
        }
    };
    run.target_triple = env!("NEXTENGINE_BUILD_TARGET_TRIPLE").to_owned();
    match xtask::performance::decode_build_toolchain_hex(env!("NEXTENGINE_BUILD_TOOLCHAIN_HEX")) {
        Ok(toolchain) => run.toolchain = toolchain,
        Err(error) => run
            .diagnostics
            .push(format!("PERF_BUILD_TOOLCHAIN_INVALID: {error}")),
    }
    if let Err(diagnostics) = run.validate_build_provenance() {
        run.diagnostics.extend(diagnostics);
    }

    match Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root)
        .output()
    {
        Ok(output) if output.status.success() => {
            let runtime_commit = String::from_utf8_lossy(&output.stdout).trim().to_owned();
            if runtime_commit != run.commit {
                run.diagnostics.push(format!(
                    "PERF_RUNTIME_COMMIT_MISMATCH: build={} runtime={runtime_commit}",
                    run.commit
                ));
            }
        }
        Ok(output) => run.diagnostics.push(format!(
            "PERF_RUNTIME_COMMIT_UNAVAILABLE: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )),
        Err(error) => run
            .diagnostics
            .push(format!("PERF_RUNTIME_COMMIT_UNAVAILABLE: {error}")),
    }
    match Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(root)
        .output()
    {
        Ok(output) if output.status.success() => {
            let runtime_worktree_clean = output.stdout.is_empty();
            if runtime_worktree_clean != run.worktree_clean {
                run.diagnostics.push(format!(
                    "PERF_RUNTIME_WORKTREE_STATE_MISMATCH: build={} runtime={runtime_worktree_clean}",
                    run.worktree_clean
                ));
            }
        }
        Ok(output) => run.diagnostics.push(format!(
            "PERF_RUNTIME_WORKTREE_STATE_UNAVAILABLE: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )),
        Err(error) => run
            .diagnostics
            .push(format!("PERF_RUNTIME_WORKTREE_STATE_UNAVAILABLE: {error}")),
    }
    match Command::new("rustc").arg("-vV").output() {
        Ok(output) if output.status.success() => {
            let runtime_toolchain = String::from_utf8_lossy(&output.stdout).trim().to_owned();
            if runtime_toolchain != run.toolchain {
                run.diagnostics
                    .push("PERF_RUNTIME_TOOLCHAIN_MISMATCH".to_owned());
            }
        }
        Ok(output) => run.diagnostics.push(format!(
            "PERF_RUNTIME_TOOLCHAIN_UNAVAILABLE: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )),
        Err(error) => run
            .diagnostics
            .push(format!("PERF_RUNTIME_TOOLCHAIN_UNAVAILABLE: {error}")),
    }
}

fn populate_performance_host(
    request: &PerformanceArguments,
    run: &mut xtask::performance::PerformanceRunV5,
) {
    let target_id = request.target.as_deref().unwrap_or("observed-host-v1");
    match xtask::performance::inspect_current_host(target_id) {
        Ok((fingerprint, preflight)) => {
            run.target_fingerprint = Some(fingerprint);
            run.preflight = Some(preflight.clone());
            run.environment_samples.push(preflight);
        }
        Err(error) => run.diagnostics.push(error),
    }
}

fn validate_gate_prerequisites(
    request: &PerformanceArguments,
    run: &mut xtask::performance::PerformanceRunV5,
) {
    if run.build_profile != "release" {
        run.diagnostics
            .push("PERF_GATE_REQUIRES_RELEASE_BUILD".to_owned());
    }
    if !run.worktree_clean {
        run.diagnostics
            .push("PERF_GATE_REQUIRES_CLEAN_COMMIT".to_owned());
    }
    if request.target.as_deref() != Some(xtask::performance::THOTH_TARGET_ID) {
        run.diagnostics
            .push("PERF_GATE_TARGET_UNSUPPORTED".to_owned());
    }
    match &run.target_fingerprint {
        Some(fingerprint) => run
            .diagnostics
            .extend(xtask::performance::validate_thoth_fingerprint(fingerprint)),
        None => run
            .diagnostics
            .push("PERF_TARGET_FINGERPRINT_UNAVAILABLE".to_owned()),
    }
    match &run.preflight {
        Some(preflight) => {
            if let Err(diagnostics) = preflight.validate_ready_evidence() {
                run.diagnostics.extend(diagnostics);
            }
        }
        None => run
            .diagnostics
            .push("PERF_PREFLIGHT_UNAVAILABLE".to_owned()),
    }
    if request.baseline.is_none() {
        run.diagnostics
            .push("PERF_GATE_BASELINE_REQUIRED".to_owned());
    }
}

#[cfg(test)]
mod tests;
