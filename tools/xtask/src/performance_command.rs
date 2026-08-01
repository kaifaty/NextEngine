#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use xtask::report::*;

use crate::run_tool_session;

mod support;

use support::*;

const INTERACTIVE_FRAME_SOAK_FRAMES: u32 = 240;
mod production_worker;

use production_worker::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PerformanceArguments {
    scenario: xtask::performance::PerformanceScenarioV1,
    mode: xtask::performance::PerformanceModeV1,
    target: Option<String>,
    baseline: Option<PathBuf>,
    output: Option<PathBuf>,
}

impl Default for PerformanceArguments {
    fn default() -> Self {
        Self {
            scenario: xtask::performance::PerformanceScenarioV1::Smoke,
            mode: xtask::performance::PerformanceModeV1::Report,
            target: None,
            baseline: None,
            output: None,
        }
    }
}

fn performance_scenario_hash(scenario: xtask::performance::PerformanceScenarioV1) -> String {
    let preimage: &[u8] = match scenario {
        xtask::performance::PerformanceScenarioV1::Smoke => {
            b"nextengine.performance.smoke.v2:two-chunk:five-object:one-agent:900-live-ticks"
        }
        xtask::performance::PerformanceScenarioV1::LongSessionSoak => {
            b"nextengine.performance.long-session-soak.v3:3600-live-ticks:1200-tick-windows:held-movement:camera-every-15-ticks:driver-and-interactive-application:one-fixed-step-per-measured-pump"
        }
        xtask::performance::PerformanceScenarioV1::InteractiveFrameSoak => {
            b"nextengine.performance.interactive-frame-soak.v1:240-fifo-frames:1920x1080:reference-render-inputs:phase-timings:frame-plan-cache"
        }
        xtask::performance::PerformanceScenarioV1::ProductionWorkerSoak => {
            b"nextengine.performance.production-worker-soak.v1:240-fifo-main-callbacks:60hz:bounded-sync-queue:next-simulation-worker:fixed-step-application:shared-presentation-publication:main-snapshot-read"
        }
        _ => return xtask::performance::sha256_hex(scenario.as_str().as_bytes()),
    };
    xtask::performance::sha256_hex(preimage)
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
    let mut run = xtask::performance::PerformanceRunV1::empty(
        request.scenario,
        request.mode,
        env!("NEXTENGINE_BUILD_PROFILE"),
    );
    run.scenario_hash = performance_scenario_hash(request.scenario);
    populate_performance_identity(root, &mut run);
    populate_performance_host(request, &mut run);

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
    let resource_counters_before = xtask::performance::inspect_process_counters();
    let (tool_run, _) = run_tool_session("tools-performance", state_root)?;
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

    let started = Instant::now();
    let streaming = match state_root {
        Some(root) => next_verification::run_streaming_performance_check_in(root),
        None => next_verification::run_streaming_performance_check(),
    }
    .map_err(|error| error.to_string())?;
    instrumentation_overhead_nanoseconds =
        instrumentation_overhead_nanoseconds.saturating_add(record_performance_span(
            profiling_enabled,
            &mut recorded_spans,
            &mut dropped_spans,
            "streaming-io",
            started.elapsed(),
        ));

    let started = Instant::now();
    let agent = match state_root {
        Some(root) => next_verification::run_agent_planning_performance_check_in(root),
        None => next_verification::run_agent_planning_performance_check(),
    }
    .map_err(|error| error.to_string())?;
    instrumentation_overhead_nanoseconds =
        instrumentation_overhead_nanoseconds.saturating_add(record_performance_span(
            profiling_enabled,
            &mut recorded_spans,
            &mut dropped_spans,
            "agent-planning",
            started.elapsed(),
        ));

    let started = Instant::now();
    let render_planning = match state_root {
        Some(root) => next_verification::run_render_frame_planning_performance_check_in(root),
        None => next_verification::run_render_frame_planning_performance_check(),
    }
    .map_err(|error| error.to_string())?;
    instrumentation_overhead_nanoseconds =
        instrumentation_overhead_nanoseconds.saturating_add(record_performance_span(
            profiling_enabled,
            &mut recorded_spans,
            &mut dropped_spans,
            "render-extraction",
            started.elapsed(),
        ));

    let started = Instant::now();
    let live_runtime = run_live_runtime_scenario(request.scenario, state_root)
        .map_err(|error| error.to_string())?;
    instrumentation_overhead_nanoseconds =
        instrumentation_overhead_nanoseconds.saturating_add(record_performance_span(
            profiling_enabled,
            &mut recorded_spans,
            &mut dropped_spans,
            "runtime-stages",
            started.elapsed(),
        ));

    let production_worker =
        if request.scenario == xtask::performance::PerformanceScenarioV1::ProductionWorkerSoak {
            Some(run_production_worker_scenario(state_root)?)
        } else {
            None
        };

    let desktop_frame_timing_requested = profiling_enabled
        || request.scenario == xtask::performance::PerformanceScenarioV1::InteractiveFrameSoak;
    let desktop_frame_timing = if desktop_frame_timing_requested {
        let started = Instant::now();
        let report = match (request.scenario, state_root) {
            (xtask::performance::PerformanceScenarioV1::InteractiveFrameSoak, Some(root)) => {
                next_verification::run_desktop_frame_timing_workload_in(
                    root,
                    INTERACTIVE_FRAME_SOAK_FRAMES,
                    [1_920, 1_080],
                )
            }
            (xtask::performance::PerformanceScenarioV1::InteractiveFrameSoak, None) => {
                next_verification::run_desktop_frame_timing_workload_in(
                    &std::env::temp_dir(),
                    INTERACTIVE_FRAME_SOAK_FRAMES,
                    [1_920, 1_080],
                )
            }
            (_, Some(root)) => next_verification::run_desktop_frame_timing_smoke_in(root),
            (_, None) => next_verification::run_desktop_frame_timing_smoke(),
        }
        .map_err(|error| error.to_string())?;
        if profiling_enabled {
            instrumentation_overhead_nanoseconds = instrumentation_overhead_nanoseconds
                .saturating_add(record_performance_span(
                    profiling_enabled,
                    &mut recorded_spans,
                    &mut dropped_spans,
                    "render-extraction",
                    started.elapsed(),
                ));
        }
        report
    } else {
        None
    };

    let authoritative_hashes = scenario_authoritative_hashes(
        &streaming,
        &agent,
        &render_planning,
        &live_runtime,
        production_worker.as_ref(),
    );
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
    run.resource_counters = xtask::performance::finish_process_counters(&resource_counters_before);
    if let Some(frame_timing) = &desktop_frame_timing {
        run.resource_counters.vulkan_timestamp_queries = frame_timing.timestamp_query_count;
        run.resource_counters.device_resident_bytes = Some(frame_timing.device_allocation_bytes);
        run.resource_counters.unavailable.retain(|diagnostic| {
            !diagnostic.starts_with("Vulkan timestamps require")
                && !diagnostic.starts_with("device residency requires")
        });
        run.methodology.notes.push(format!(
            "{}-frame Vulkan timing workload uses production render inputs at {}x{} drawable extent and remains report-only",
            frame_timing.samples.len(),
            frame_timing.drawable_extent[0],
            frame_timing.drawable_extent[1],
        ));
        run.methodology.notes.push(format!(
            "device residency uses a conservative ceiling of {} engine-owned bound Vulkan allocations; swapchain storage is driver-owned",
            frame_timing.device_allocation_count
        ));
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
                "long-session-soak.application.checkpoint",
                "microseconds",
                live_runtime
                    .application_checkpoint_microseconds
                    .into_iter()
                    .map(microseconds_u64)
                    .collect::<Result<Vec<_>, _>>()?,
                None,
            )?);
        run.metrics
            .push(xtask::performance::PerformanceMetricV1::from_samples(
                "long-session-soak.application.ordinary-tick",
                "microseconds",
                live_runtime
                    .application_ordinary_tick_microseconds
                    .iter()
                    .copied()
                    .map(microseconds_u64)
                    .collect::<Result<Vec<_>, _>>()?,
                None,
            )?);
        run.metrics
            .push(xtask::performance::PerformanceMetricV1::from_samples(
                "long-session-soak.application.checkpoint-tick",
                "microseconds",
                live_runtime
                    .application_checkpoint_tick_microseconds
                    .iter()
                    .copied()
                    .map(microseconds_u64)
                    .collect::<Result<Vec<_>, _>>()?,
                None,
            )?);
    }
    if let Some(worker) = &production_worker {
        append_production_worker_metrics(&mut run.metrics, worker)?;
        run.methodology.notes.push(format!(
            "observed {} fixed steps ({} ordinary, {} durable-checkpoint), {} shared snapshot publications including the initial generation, and queue high-water {}",
            worker.metrics.fixed_steps,
            worker.metrics.ordinary_fixed_steps,
            worker.metrics.checkpoint_fixed_steps,
            worker.metrics.snapshot_publications,
            worker.metrics.queue_high_water,
        ));
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
    if let Err(error) = run.instrumentation.validate() {
        run.diagnostics.push(error);
        run.verdict = xtask::performance::PerformanceVerdict::NotRun;
    }

    if run.verdict != xtask::performance::PerformanceVerdict::NotRun
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

struct ProfilerControl {
    authoritative_hashes: BTreeMap<String, String>,
}

fn run_profiler_control(
    scenario: xtask::performance::PerformanceScenarioV1,
    state_root: Option<&Path>,
) -> Result<ProfilerControl, String> {
    let streaming = match state_root {
        Some(root) => next_verification::run_streaming_performance_check_in(root),
        None => next_verification::run_streaming_performance_check(),
    }
    .map_err(|error| error.to_string())?;
    let agent = match state_root {
        Some(root) => next_verification::run_agent_planning_performance_check_in(root),
        None => next_verification::run_agent_planning_performance_check(),
    }
    .map_err(|error| error.to_string())?;
    let render = match state_root {
        Some(root) => next_verification::run_render_frame_planning_performance_check_in(root),
        None => next_verification::run_render_frame_planning_performance_check(),
    }
    .map_err(|error| error.to_string())?;
    let live =
        run_live_runtime_scenario(scenario, state_root).map_err(|error| error.to_string())?;
    let production_worker =
        if scenario == xtask::performance::PerformanceScenarioV1::ProductionWorkerSoak {
            Some(run_production_worker_scenario(state_root)?)
        } else {
            None
        };
    Ok(ProfilerControl {
        authoritative_hashes: scenario_authoritative_hashes(
            &streaming,
            &agent,
            &render,
            &live,
            production_worker.as_ref(),
        ),
    })
}

fn run_live_runtime_scenario(
    scenario: xtask::performance::PerformanceScenarioV1,
    state_root: Option<&Path>,
) -> Result<
    next_verification::LiveRuntimePerformanceReport,
    next_verification::LiveRuntimePerformanceError,
> {
    match (scenario, state_root) {
        (
            xtask::performance::PerformanceScenarioV1::Smoke
            | xtask::performance::PerformanceScenarioV1::InteractiveFrameSoak
            | xtask::performance::PerformanceScenarioV1::ProductionWorkerSoak,
            Some(root),
        ) => next_verification::run_live_runtime_performance_check_in(root),
        (
            xtask::performance::PerformanceScenarioV1::Smoke
            | xtask::performance::PerformanceScenarioV1::InteractiveFrameSoak
            | xtask::performance::PerformanceScenarioV1::ProductionWorkerSoak,
            None,
        ) => next_verification::run_live_runtime_performance_check(),
        (xtask::performance::PerformanceScenarioV1::LongSessionSoak, Some(root)) => {
            next_verification::run_live_runtime_long_session_performance_check_in(root)
        }
        (xtask::performance::PerformanceScenarioV1::LongSessionSoak, None) => {
            next_verification::run_live_runtime_long_session_performance_check()
        }
        _ => unreachable!("unavailable representative scenarios return before execution"),
    }
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

fn populate_performance_identity(root: &Path, run: &mut xtask::performance::PerformanceRunV1) {
    match Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root)
        .output()
    {
        Ok(output) if output.status.success() => {
            run.commit = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        }
        Ok(output) => run.diagnostics.push(format!(
            "PERF_COMMIT_UNAVAILABLE: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )),
        Err(error) => run
            .diagnostics
            .push(format!("PERF_COMMIT_UNAVAILABLE: {error}")),
    }
    match Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(root)
        .output()
    {
        Ok(output) if output.status.success() => {
            run.worktree_clean = output.stdout.is_empty();
        }
        Ok(output) => run.diagnostics.push(format!(
            "PERF_WORKTREE_STATE_UNAVAILABLE: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )),
        Err(error) => run
            .diagnostics
            .push(format!("PERF_WORKTREE_STATE_UNAVAILABLE: {error}")),
    }
    match Command::new("rustc").arg("-vV").output() {
        Ok(output) if output.status.success() => {
            run.toolchain = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        }
        Ok(output) => run.diagnostics.push(format!(
            "PERF_TOOLCHAIN_UNAVAILABLE: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )),
        Err(error) => run
            .diagnostics
            .push(format!("PERF_TOOLCHAIN_UNAVAILABLE: {error}")),
    }
}

fn populate_performance_host(
    request: &PerformanceArguments,
    run: &mut xtask::performance::PerformanceRunV1,
) {
    let target_id = request.target.as_deref().unwrap_or("observed-host-v1");
    match xtask::performance::inspect_current_host(target_id) {
        Ok((fingerprint, preflight)) => {
            run.target_fingerprint = Some(fingerprint);
            run.preflight = Some(preflight);
        }
        Err(error) => run.diagnostics.push(error),
    }
}

fn validate_gate_prerequisites(
    request: &PerformanceArguments,
    run: &mut xtask::performance::PerformanceRunV1,
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
        Some(preflight) if preflight.ready => {}
        Some(preflight) => run.diagnostics.extend(preflight.diagnostics.clone()),
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
